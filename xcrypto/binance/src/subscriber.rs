use crate::chat::MarketStream;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use tokio::sync::mpsc::UnboundedSender;
use xcrypto::{
    chat::{ErrorResponse, Request, Response},
    tungstenite::Message,
};
pub struct Subscriber {
    symbols: HashSet<String>,
    tx: UnboundedSender<Message>,
    ids: HashMap<i64, i64>,
}

impl Subscriber {
    pub fn new(tx: UnboundedSender<Message>) -> Self {
        Self {
            symbols: HashSet::default(),
            tx,
            ids: HashMap::default(),
        }
    }

    pub fn on_response<T: Serialize>(&mut self, mut response: Response<T>) -> anyhow::Result<()> {
        if let Some(id) = self.ids.remove(&response.id) {
            response.id = id;
            self.tx
                .send(Message::Text(serde_json::to_string(&response)?))?;
        }
        Ok(())
    }

    pub fn on_error(&mut self, mut response: ErrorResponse) -> anyhow::Result<()> {
        if let Some(id) = self.ids.remove(&response.id) {
            response.id = id;
            self.tx
                .send(Message::Text(serde_json::to_string(&response)?))?;
        }
        Ok(())
    }

    pub fn on_subscribe(&mut self, id: i64, req: Request<Vec<String>>) {
        self.ids.insert(id, req.id);
        self.symbols.extend(
            req.params
                .into_iter()
                .map(|x| x.replace("_", ":"))
                .collect::<Vec<String>>(),
        );
    }

    pub fn is_subscribed(&self, symbol: &String) -> bool {
        self.symbols.contains(symbol)
    }

    pub fn forward(&self, stream: &mut MarketStream) -> anyhow::Result<()> {
        match stream {
            MarketStream::Depth(depth) => {
                if depth.bid(0) < depth.bid(1) {
                    depth.reverse();
                }

                if depth.bid(0) >= depth.ask(0) {
                    return Ok(());
                }

                if self.is_subscribed(&depth.stream()) {
                    self.tx
                        .send(Message::Text(serde_json::to_string(&depth)?))?;
                }
            }
            MarketStream::Kline(kline) => {
                // finished kline
                if kline.k.x && self.is_subscribed(&kline.stream()) {
                    self.tx
                        .send(Message::Text(serde_json::to_string(&kline)?))?;
                }
            }
        }
        Ok(())
    }

    pub fn iter(&self) -> std::collections::hash_set::Iter<std::string::String> {
        self.symbols.iter()
    }
}
