use binance::chat::*;
use binance::*;
use log::*;
use native_json::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::fmt::Debug;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use xcrypto::chat::*;
use xcrypto::error::*;
use xcrypto::parser::Parser;
use xcrypto::position::PositionDB;
use xcrypto::rest::Rest;
use xcrypto::tungstenite::Message;

async fn get_positions(rest: &Arc<Rest>) -> anyhow::Result<HashMap<String, BinanceProduct>> {
    let rsp = rest.get("/fapi/v1/exchangeInfo", &[], false).await?;
    let results: serde_json::Value = serde_json::from_str(&rsp.text().await?)?;

    let results = results.get("symbols").unwrap();
    let results: Vec<BinanceProduct> = serde_json::from_value(results.to_owned())?;
    let mut products = HashMap::new();

    for result in results {
        products.insert(result.symbol.clone(), result);
    }

    info!("products {}", products.len());
    Ok(products)
}

pub struct UsdtTrade {
    rest: Arc<Rest>,
    txs: HashMap<SocketAddr, UnboundedSender<Message>>,
    account: Account<UsdtListenKey>,
    // addr -> session_id
    session_id: HashMap<SocketAddr, u16>,
    // session_id -> session
    session: HashMap<u16, Session>,
    posdb: Arc<PositionDB>,
    products: HashMap<String, BinanceProduct>,
}

impl UsdtTrade {
    pub async fn new(rest: Arc<Rest>, account: Account<UsdtListenKey>) -> anyhow::Result<Self> {
        let products = get_positions(&rest).await?;

        Ok(Self {
            rest,
            txs: HashMap::default(),
            account,
            session_id: HashMap::default(),
            session: HashMap::default(),
            posdb: Arc::new(PositionDB::new("pos.db").await?),
            products,
        })
    }
}
impl Trade for UsdtTrade {
    fn disconnected(&self) -> bool {
        self.account.disconnected()
    }

    fn products(&self) -> &HashMap<String, BinanceProduct> {
        &self.products
    }

    fn get_positions(&self, session_id: u16) -> Option<&HashMap<String, Position>> {
        self.posdb.get_positions(session_id)
    }

    async fn get_products(&mut self) -> anyhow::Result<()> {
        self.products = get_positions(&self.rest).await?;
        Ok(())
    }

    async fn process(&mut self) -> anyhow::Result<bool> {
        let msg = self.account.process().await?;

        if let Some(Message::Text(s)) = msg {
            if let Event::OrderUpdate(order) = serde_json::from_str::<Event>(&s)? {
                self.on_order(&order);
            }
        }

        Ok(self.disconnected())
    }

    fn add_order(&mut self, addr: &SocketAddr, order: &BinanceOrder) -> anyhow::Result<()> {
        match self.txs.get_mut(addr) {
            Some(tx) => {
                let rest = self.rest.clone();
                let tx = tx.clone();

                let symbol = order.symbol.clone();
                let price = order.price;
                let quantity = order.quantity;
                let side = order.side.clone();
                let order_type = order.order_type.clone();
                let tif = order.tif.clone();
                let session_id = order.session_id;
                let id = order.id;

                tokio::spawn(async move {
                    match rest
                        .add_order(
                            "/fapi/v1/order",
                            symbol.clone().to_uppercase(),
                            price.to_string(),
                            quantity.to_string(),
                            format!("{:?}", side),
                            format!("{:?}", order_type),
                            format!("{:?}", tif),
                            session_id,
                            id,
                        )
                        .await
                    {
                        Ok(rsp) => {
                            // exchange rej
                            if let Ok(e) = rsp.json::<xcrypto::chat::Error>().await {
                                error!("{:?}", e);
                                let order = Order::new(
                                    id,
                                    symbol,
                                    side,
                                    State::REJECTED,
                                    order_type,
                                    tif,
                                    quantity,
                                    price,
                                );

                                match serde_json::to_string(&order) {
                                    Ok(s) => {
                                        if let Err(e) = tx.send(Message::Text(s)) {
                                            error!("{}", e);
                                        }
                                    }
                                    Err(e) => error!("{}", e),
                                }
                            }
                        }
                        // network error
                        Err(e) => {
                            error!("{:?}", e);
                            let order = Order::new(
                                id,
                                symbol,
                                side,
                                State::REJECTED,
                                order_type,
                                tif,
                                quantity,
                                price,
                            );

                            match serde_json::to_string(&order) {
                                Ok(s) => {
                                    if let Err(e) = tx.send(Message::Text(s)) {
                                        error!("{}", e);
                                    }
                                }
                                Err(e) => error!("{}", e),
                            }
                        }
                    }
                });
            }
            None => warn!("Missing session {}, maybe a bug", addr),
        }

        Ok(())
    }

    fn cancel(&mut self, addr: &SocketAddr, cancel: &BinanceCancel) -> anyhow::Result<()> {
        match self.txs.get_mut(addr) {
            Some(_) => {
                let rest = self.rest.clone();

                let symbol = cancel.symbol.clone().to_uppercase();
                let session_id = cancel.session_id;
                let order_id = cancel.order_id;

                let orig = u64::from(session_id) << 32 | u64::from(order_id);
                tokio::spawn(async move {
                    if let Err(e) = rest.cancel("/fapi/v1/order", symbol, orig).await {
                        error!("{}", e)
                    }
                });
            }
            None => warn!("Missing session {}, maybe a bug", addr),
        }
        Ok(())
    }

    fn handle_close(&mut self, addr: &SocketAddr) -> anyhow::Result<()> {
        if let Some(_) = self.txs.remove(addr) {
            match self.session_id.remove(addr) {
                Some(id) => match self.session.get_mut(&id) {
                    Some(session) => {
                        session.set_active(None);
                    }
                    None => warn!("Session used by {} isn't exist, maybe a bug", addr),
                },
                None => warn!("Session used by {} isn't exist, maybe a bug", addr),
            }
        }
        Ok(())
    }

    async fn handle_login(
        &mut self,
        addr: &SocketAddr,
        req: &Request<Login>,
        tx: &UnboundedSender<Message>,
    ) -> anyhow::Result<Option<Error>> {
        let login = &req.params;
        let session_id = login.session_id;

        match self.session.get_mut(&session_id) {
            Some(session) => {
                if session.active() {
                    return Ok(Some(Error {
                        code: DUPLICATE_LOGIN,
                        msg: "duplicate login".into(),
                    }));
                } else {
                    session.set_active(Some(tx.clone()));
                }
            }
            None => {
                let session = Session::new(session_id, self.posdb.clone(), tx.clone()).await?;
                self.session.insert(session_id, session);
            }
        }
        self.txs.insert(addr.clone(), tx.clone());
        self.session_id.insert(addr.clone(), session_id);

        info!("session addr {} -> {}", addr, session_id);
        Ok(None)
    }

    #[allow(unused)]
    fn handle_subscribe(&mut self, addr: &SocketAddr, req: &Request<Vec<String>>) -> Option<Error> {
        let mut params = Vec::new();

        for symbol in req.params.iter() {
            let symbol = symbol.to_lowercase();
            match symbol.split_once("@") {
                Some((name, stream)) => {
                    if !self.products.contains_key(name) {
                        return Some(Error {
                            code: INVALID_SYMBOL,
                            msg: format!("invalid symbol {}", symbol),
                        });
                    }
                    if !self.validate_symbol(name, stream) {
                        return Some(Error {
                            code: INVALID_STREAM,
                            msg: format!("invalid stream {}", symbol),
                        });
                    }
                }
                None => {
                    return Some(Error {
                        code: INVALID_SYMBOL,
                        msg: format!("invalid symbol {}", symbol),
                    });
                }
            }
            params.push(symbol);
        }
        None
    }

    #[allow(unused)]
    fn validate_symbol(&self, symbol: &str, stream: &str) -> bool {
        match stream.split_once(":") {
            Some((stream, interval)) => {
                if stream != "kline" {
                    return false;
                }

                match interval {
                    "1s" | "1m" | "3m" | "5m" | "15m" | "30m" | "1h" | "2h" | "4h" | "6h"
                    | "8h" | "12h" | "1d" | "3d" | "1w" | "1M" => true,
                    _ => false,
                }
            }
            None => match stream {
                "depth" | "bookicker" => true,
                _ => false,
            },
        }
    }

    fn handle_disconnect(&mut self, addr: &SocketAddr, parser: &Parser) -> anyhow::Result<()> {
        if let Some(id) = parser.get("id") {
            self.reply(
                addr,
                i64::deserialize(id)?,
                Error {
                    code: DISCONNECTED,
                    msg: "trade disconnected".into(),
                },
            )?;
        }
        Ok(())
    }
    fn reply<T: Serialize + Debug>(
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

    async fn reconncet(&mut self) -> anyhow::Result<()> {
        self.account.reconnect().await
    }
}

// callback
impl UsdtTrade {
    fn on_order(&mut self, order: &OrderUpdate) {
        info!("{:?}", order);
        let client_order_id = order.o.c.parse::<u64>();

        match client_order_id {
            Ok(client_order_id) => {
                let session_id = (client_order_id >> 32) as u16;

                match self.session.get_mut(&session_id) {
                    Some(session) => {
                        if let Err(e) = session.on_order(order) {
                            error!("{}", e);
                        }
                    }
                    None => warn!("Missing session {}, maybe a bug", session_id),
                }
            }
            Err(_) => info!("Extrnal order:{:?} ", order),
        }
    }
}
