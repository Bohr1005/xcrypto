pub mod account;
pub mod app;
pub mod chat;
pub mod handler;
pub mod market;
pub mod session;
pub mod subscriber;

pub use account::*;
pub use app::*;
pub use chat::*;
pub use handler::*;
pub use market::*;
pub use session::*;
use std::future::Future;
pub use subscriber::*;

use serde::Serialize;
use std::collections::HashMap;
use std::fmt::Debug;
use std::net::SocketAddr;
use tokio::sync::mpsc::UnboundedSender;
use xcrypto::chat::*;
use xcrypto::parser::Parser;
use xcrypto::tungstenite::Message;

pub trait Trade {
    fn disconnected(&self) -> bool;
    fn products(&self) -> &HashMap<String, BinanceProduct>;
    fn get_positions(&self, session_id: u16) -> Option<&HashMap<String, Position>>;
    fn get_products(&mut self) -> impl Future<Output = anyhow::Result<()>> + Send;
    fn process(&mut self) -> impl Future<Output = anyhow::Result<bool>> + Send;
    fn add_order(&mut self, addr: &SocketAddr, order: &BinanceOrder) -> anyhow::Result<()>;
    fn cancel(&mut self, addr: &SocketAddr, cancel: &BinanceCancel) -> anyhow::Result<()>;
    fn handle_close(&mut self, addr: &SocketAddr) -> anyhow::Result<()>;
    fn handle_login(
        &mut self,
        addr: &SocketAddr,
        req: &Request<Login>,
        tx: &UnboundedSender<Message>,
    ) -> impl Future<Output = anyhow::Result<Option<Error>>> + Send;
    fn handle_subscribe(&mut self, addr: &SocketAddr, req: &Request<Vec<String>>) -> Option<Error>;
    fn validate_symbol(&self, symbol: &str, stream: &str) -> bool;
    fn handle_disconnect(&mut self, addr: &SocketAddr, parser: &Parser) -> anyhow::Result<()>;
    fn reply<T: Serialize + Debug>(
        &mut self,
        addr: &SocketAddr,
        id: i64,
        result: T,
    ) -> anyhow::Result<()>;
    fn reconncet(&mut self) -> impl Future<Output = anyhow::Result<()>> + Send;
}

pub trait OrderTrait {
    fn symbol(&self) -> &str;
    fn trd_vol(&self) -> anyhow::Result<f64>;
    fn commission(&self) -> f64;
    fn net(&self) -> anyhow::Result<f64>;
    fn side(&self) -> Side;
    fn state(&self) -> State;
}

pub trait ListenKey {
    fn key(&self) -> &str;
}
