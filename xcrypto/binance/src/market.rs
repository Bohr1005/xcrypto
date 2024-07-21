use crate::chat::Event;
use crate::{BinanceQuote, MarketStream, Subscriber, Trade};
use log::*;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::{collections::HashMap, fmt::Debug};
use tokio::sync::mpsc::UnboundedSender;
use tokio::time::{Duration, Instant};
use xcrypto::parser::Parser;
use xcrypto::ws::WebSocket;
use xcrypto::{chat::*, error::*, tungstenite::Message};

pub struct Market {
    addr: String,
    txs: HashMap<SocketAddr, UnboundedSender<Message>>,
    subscribers: HashMap<SocketAddr, Subscriber>,
    symbols: HashMap<String, u16>,
    requests: HashMap<i64, SocketAddr>,
    ws: WebSocket,
    disconnected: bool,
    id: i64,
    time: Instant,
}

impl Market {
    pub async fn new(addr: String) -> anyhow::Result<Self> {
        let mut ws = WebSocket::client(&addr).await?;
        let combined = r#"{"method": "SET_PROPERTY","params": ["combined", true],"id": 0}"#;
        ws.send(Message::text(combined.to_string())).await?;

        Ok(Self {
            addr,
            txs: HashMap::default(),
            subscribers: HashMap::default(),
            symbols: HashMap::default(),
            requests: HashMap::default(),
            ws,
            disconnected: false,
            id: 1,
            time: Instant::now(),
        })
    }

    pub fn disconnected(&self) -> bool {
        self.disconnected
    }

    async fn subscribe(&mut self, symbols: Vec<String>) -> anyhow::Result<()> {
        let req: Request<Vec<String>> = Request {
            id: self.id,
            method: "SUBSCRIBE".into(),
            params: symbols,
        };
        info!("{:?}", req);
        self.ws
            .send(Message::Text(serde_json::to_string(&req)?))
            .await?;
        self.id += 1;
        Ok(())
    }

    async fn send<T: Serialize + Debug>(
        &mut self,
        addr: &SocketAddr,
        method: String,
        param: T,
    ) -> anyhow::Result<i64> {
        let req: Request<T> = Request {
            id: self.id,
            method: method,
            params: param,
        };

        info!("{:?}", req);
        let msg = Message::Text(serde_json::to_string(&req)?);
        self.ws.send(msg).await?;

        self.requests.insert(req.id, addr.clone());
        self.id += 1;

        Ok(req.id)
    }

    pub fn reply<T: Serialize + Debug>(
        &mut self,
        addr: &SocketAddr,
        id: i64,
        result: T,
    ) -> anyhow::Result<()> {
        if let Some(tx) = self.txs.get_mut(addr) {
            let response = Response {
                id: id,
                result: result,
            };

            debug!("{:?}", response);
            let rsp = Message::Text(serde_json::to_string(&response)?);
            tx.send(rsp)?;
        }
        Ok(())
    }

    pub async fn reconncet<T: Trade>(&mut self, trade: &mut T) -> anyhow::Result<()> {
        if !self.disconnected {
            return Ok(());
        }

        if self.time.elapsed() >= Duration::from_secs(30) {
            info!("Reconnecting to {}", self.addr);
            self.time = Instant::now();

            match WebSocket::client(&self.addr).await {
                Ok(mut ws) => {
                    let combined =
                        r#"{"method": "SET_PROPERTY","params": ["combined", true],"id": 0}"#;
                    ws.send(Message::text(combined.to_string())).await?;

                    self.ws = ws;
                    self.disconnected = false;

                    // re-subscribe
                    if !self.symbols.is_empty() {
                        let symbols: Vec<_> = self
                            .symbols
                            .keys()
                            .cloned()
                            .map(|x| x.replace(":", "_"))
                            .collect();
                        self.subscribe(symbols).await?;
                    }
                    trade.get_products().await?
                }
                Err(e) => error!("{}", e),
            }
        }
        // release cpu
        tokio::time::sleep(Duration::ZERO).await;
        Ok(())
    }

    pub async fn process(&mut self) -> anyhow::Result<bool> {
        match self.ws.recv().await? {
            Some(inner) => match &inner {
                Message::Text(s) => match serde_json::from_str::<Event>(s) {
                    Ok(e) => self.handle_event(e),
                    Err(e) => error!("{} {}", e, inner),
                },
                Message::Ping(ping) => {
                    debug!("{:?}", inner);
                    self.ws.send(Message::Pong(ping.to_owned())).await?;
                }
                _ => (),
            },
            None => {
                if !self.disconnected {
                    error!("market disconnected");
                    self.disconnected = true
                }
            }
        }
        Ok(self.disconnected)
    }
}

// handler
impl Market {
    pub async fn handle_close(&mut self, addr: &SocketAddr) -> anyhow::Result<()> {
        if let Some(_) = self.txs.remove(addr) {
            let mut unsubscribe = Vec::new();
            let val = self.subscribers.remove(addr);
            match &val {
                Some(subscriber) => {
                    info!("Bye subscriber {}", addr);
                    // unsubscribe
                    for symbol in subscriber.iter() {
                        if let Some(cnt) = self.symbols.get_mut(symbol) {
                            *cnt -= 1;
                            if *cnt == 0 {
                                if let Some(_) = self.symbols.remove(symbol) {
                                    info!("Unsubscribe {}", symbol);
                                    unsubscribe.push(symbol.replace(":", "_"));
                                }
                            }
                        }
                    }
                }
                None => info!("Subscriber({}) isn't login", addr),
            }

            if !unsubscribe.is_empty() {
                self.send(addr, "UNSUBSCRIBE".into(), unsubscribe).await?;
            }
        }
        Ok(())
    }

    fn handle_error(&mut self, err: ErrorResponse) {
        if let Some(index) = self.requests.remove(&err.id) {
            if let Some(subscriber) = self.subscribers.get_mut(&index) {
                if let Err(e) = subscriber.on_error(err) {
                    error!("{}", e);
                }
            }
        }
    }

    fn handle_success(&mut self, suc: Response<Option<i64>>) {
        if let Some(index) = self.requests.remove(&suc.id) {
            if let Some(subscriber) = self.subscribers.get_mut(&index) {
                if let Err(e) = subscriber.on_response(suc) {
                    error!("{}", e);
                }
            }
        }
    }

    fn handle_stream(&mut self, stream: MarketStream) -> anyhow::Result<()> {
        let s = match &stream {
            MarketStream::BookTicker(book) => book.stream().clone(),
            MarketStream::Kline(kline) => kline.stream().clone(),
            MarketStream::SpotDepth(depth) => depth.stream().clone(),
            MarketStream::FutureDepth(depth) => depth.stream().clone(),
        };

        let data = match stream {
            MarketStream::BookTicker(book) => {
                let depth: Depth<BinanceQuote> = book.into();
                serde_json::to_string(&depth)?
            }
            MarketStream::Kline(kline) => {
                let kline: Kline = kline.into();
                serde_json::to_string(&kline)?
            }
            MarketStream::SpotDepth(depth) => {
                let depth: Depth<BinanceQuote> = depth.into();
                serde_json::to_string(&depth)?
            }
            MarketStream::FutureDepth(depth) => {
                let depth: Depth<BinanceQuote> = depth.into();
                serde_json::to_string(&depth)?
            }
        };

        for subscriber in self.subscribers.values_mut() {
            if subscriber.is_subscribed(&s) {
                if let Err(e) = subscriber.forward(&data) {
                    error!("{}", e);
                }
            }
        }

        Ok(())
    }

    pub fn handle_connect(&mut self, addr: &SocketAddr, tx: &UnboundedSender<Message>) {
        self.txs.insert(addr.clone(), tx.clone());
    }

    pub fn handle_login(&mut self, addr: &SocketAddr, req: &Request<Login>) -> anyhow::Result<()> {
        if let Some(tx) = self.txs.get_mut(addr) {
            if !self.subscribers.contains_key(addr) {
                info!("New subscriber {}", addr);
                self.subscribers
                    .insert(addr.clone(), Subscriber::new(tx.clone()));
            }
        }
        self.reply(addr, req.id, req.params.clone())
    }

    pub async fn handle_subscribe(
        &mut self,
        addr: &SocketAddr,
        req: &mut Request<Vec<String>>,
    ) -> anyhow::Result<()> {
        if !self.validate_login(addr) {
            return self.reply(
                addr,
                req.id,
                Error {
                    code: NOT_LOGIN,
                    msg: "please login first".into(),
                },
            );
        }

        if let Some(subscriber) = self.subscribers.get_mut(addr) {
            let mut symbols = Vec::new();
            for symbol in req.params.iter() {
                if subscriber.is_subscribed(symbol) {
                    continue;
                }

                let symbol = if symbol.contains("kline") {
                    symbol.replace(":", "_")
                } else if symbol.contains("bbo") {
                    symbol.replace("bbo", "bookTicker")
                } else if symbol.contains("depth") {
                    symbol.replace("depth", "depth20").replace(":", "@")
                } else {
                    symbol.clone()
                };

                match self.symbols.get_mut(&symbol) {
                    Some(cnt) => *cnt += 1,
                    None => {
                        self.symbols.insert(symbol.clone(), 1);
                    }
                }

                symbols.push(symbol);
            }

            let id = self.send(addr, "SUBSCRIBE".into(), symbols.clone()).await?;
            if let Some(subscriber) = self.subscribers.get_mut(addr) {
                subscriber.on_subscribe(id, req.id, symbols);
            }
        }

        Ok(())
    }

    fn handle_event(&mut self, event: Event) {
        debug!("{:?}", event);
        match event {
            Event::Success(suc) => self.handle_success(suc),
            Event::Error(e) => self.handle_error(e),
            Event::Stream(stream) => {
                if let Err(e) = self.handle_stream(stream) {
                    error!("{}", e)
                }
            }
            _ => (),
        }
    }

    pub fn handle_disconnect(&mut self, addr: &SocketAddr, parser: &Parser) -> anyhow::Result<()> {
        if let Some(id) = parser.get("id") {
            self.reply(
                addr,
                i64::deserialize(id)?,
                Error {
                    code: DISCONNECTED,
                    msg: "market disconnected".into(),
                },
            )?;
        }
        Ok(())
    }
}

impl Market {
    fn validate_login(&self, addr: &SocketAddr) -> bool {
        self.subscribers.contains_key(addr)
    }
}
