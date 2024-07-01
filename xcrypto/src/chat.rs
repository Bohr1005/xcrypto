use native_json::{is_default, json};
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, str::FromStr};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Request<T> {
    pub id: i64,
    pub method: String,
    pub params: T,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Response<T> {
    pub id: i64,
    pub result: T,
}

json! {
    PositionReq {
        session_id: u16,
        symbols: Vec<String>,
    }
}

json! {
    PositionRsp {
        session_id: u16,
        positions: Vec<Position>,
    }
}

json! {
    Error {
    code: i32,
    msg: String,
    }
}

json! {
Login {
    session_id: u16,
    name: String?,
    trading: bool,
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Depth<T> {
    pub time: i64,
    pub symbol: String,
    pub stream: String,
    pub bids: Vec<T>,
    pub asks: Vec<T>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Kline {
    pub time: i64,
    pub symbol: String,
    pub stream: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub amount: f64,
}

#[derive(Debug, Serialize)]
pub struct Order {
    pub time: i64,
    pub symbol: String,
    pub side: Side,
    pub state: State,
    pub order_type: OrderType,
    pub tif: Tif,
    pub quantity: f64,
    pub price: f64,
    pub order_id: i64,
    pub internal_id: u32,
    pub trade_time: i64,
    pub trade_price: f64,
    pub trade_quantity: f64,
    pub acc: f64,
    pub making: bool,
}

impl Order {
    pub fn new(
        id: u32,
        symbol: String,
        side: Side,
        state: State,
        order_type: OrderType,
        tif: Tif,
        quantity: f64,
        price: f64,
    ) -> Self {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;
        Self {
            time: ts,
            symbol,
            side,
            state,
            order_type,
            tif,
            quantity,
            price,
            order_id: -1,
            internal_id: id,
            trade_time: 0,
            trade_price: 0.0,
            trade_quantity: 0.0,
            acc: 0.0,
            making: false,
        }
    }
}
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[allow(non_camel_case_types)]
pub enum State {
    NEW,
    PARTIALLY_FILLED,
    FILLED,
    CANCELED,
    REJECTED,
    EXPIRED,
    #[allow(non_camel_case_types)]
    EXPIRED_IN_MATCH,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum Side {
    BUY,
    SELL,
}

impl FromStr for Side {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "BUY" => Ok(Self::BUY),
            "SELL" => Ok(Self::SELL),
            _ => unreachable!(),
        }
    }
}
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
#[allow(non_camel_case_types)]
pub enum OrderType {
    LIMIT,
    LIMIT_MAKER,
    MARKET,
    STOP,
    STOP_MARKET,
    STOP_LOSS,
    STOP_LOSS_LIMIT,
    TAKE_PROFIT,
    TAKE_PROFIT_LIMIT,
    TAKE_PROFIT_MARKET,
    TRAILING_STOP_MARKET,
}

impl FromStr for OrderType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "LIMIT" => Ok(OrderType::LIMIT),
            "LIMIT_MAKER" => Ok(OrderType::LIMIT_MAKER),
            "MARKET" => Ok(OrderType::MARKET),
            "STOP" => Ok(OrderType::STOP),
            "STOP_MARKET" => Ok(OrderType::STOP_MARKET),
            "STOP_LOSS" => Ok(OrderType::STOP_LOSS),
            "STOP_LOSS_LIMIT" => Ok(OrderType::STOP_LOSS_LIMIT),
            "TAKE_PROFIT" => Ok(OrderType::TAKE_PROFIT),
            "TAKE_PROFIT_LIMIT" => Ok(OrderType::TAKE_PROFIT_LIMIT),
            "TAKE_PROFIT_MARKET" => Ok(OrderType::TRAILING_STOP_MARKET),
            "TRAILING_STOP_MARKET" => Ok(OrderType::TRAILING_STOP_MARKET),
            _ => unreachable!(),
        }
    }
}
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub enum Tif {
    GTC,
    IOC,
    FOK,
    GTX,
    GTD,
    UNDEF,
}

impl FromStr for Tif {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GTC" => Ok(Self::GTC),
            "IOC" => Ok(Self::IOC),
            "FOK" => Ok(Self::FOK),
            "GTX" => Ok(Self::GTX),
            "GTD" => Ok(Self::GTD),
            _ => Ok(Self::UNDEF),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Position {
    pub symbol: String,
    pub net: f64,
}

pub type Success = Response<Option<u8>>;
pub type LoginResponse = Response<Login>;
pub type ErrorResponse = Response<Error>;
