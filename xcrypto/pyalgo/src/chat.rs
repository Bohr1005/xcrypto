use crate::constant::*;
use chrono::DateTime;
use chrono_tz::{Asia::Shanghai, Tz};
use pyo3::prelude::*;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashSet;
use xcrypto::chat::{ErrorResponse, LoginResponse, Response, Success};

#[derive(Debug, Deserialize)]
struct Quote {
    pub price: f64,
    pub quantity: f64,
}

#[derive(Debug, Deserialize)]
#[pyclass]
pub struct Depth {
    time: u64,
    symbol: String,
    stream: String,
    bids: Vec<Quote>,
    asks: Vec<Quote>,
}

#[pymethods]
impl Depth {
    #[getter]
    fn time(&self) -> u64 {
        self.time
    }

    #[getter]
    fn datetime(&self) -> DateTime<Tz> {
        DateTime::from_timestamp_millis(self.time as i64)
            .unwrap()
            .with_timezone(&Shanghai)
    }

    #[getter]
    fn symbol(&self) -> &String {
        &self.symbol
    }

    #[getter]
    fn stream(&self) -> &String {
        &self.stream
    }

    #[getter]
    fn bid_level(&self) -> usize {
        self.bids.len()
    }

    #[getter]
    fn ask_level(&self) -> usize {
        self.asks.len()
    }

    fn bid_prc(&self, level: usize) -> f64 {
        match self.bids.get(level) {
            Some(quote) => quote.price,
            None => 0.0,
        }
    }

    fn bid_vol(&self, level: usize) -> f64 {
        match self.bids.get(level) {
            Some(quote) => quote.quantity,
            None => 0.0,
        }
    }

    fn ask_prc(&self, level: usize) -> f64 {
        match self.asks.get(level) {
            Some(quote) => quote.price,
            None => 0.0,
        }
    }

    fn ask_vol(&self, level: usize) -> f64 {
        match self.asks.get(level) {
            Some(quote) => quote.quantity,
            None => 0.0,
        }
    }
}

#[derive(Debug, Deserialize)]
#[pyclass]
pub struct Kline {
    time: u64,
    symbol: String,
    stream: String,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
    amount: f64,
}

#[pymethods]
impl Kline {
    #[getter]
    fn time(&self) -> u64 {
        self.time
    }

    #[getter]
    fn datetime(&self) -> DateTime<Tz> {
        DateTime::from_timestamp_millis(self.time as i64)
            .unwrap()
            .with_timezone(&Shanghai)
    }

    #[getter]
    fn symbol(&self) -> &String {
        &self.symbol
    }

    #[getter]
    fn stream(&self) -> &String {
        &self.stream
    }

    #[getter]
    fn open(&self) -> f64 {
        self.open
    }

    #[getter]
    fn high(&self) -> f64 {
        self.high
    }

    #[getter]
    fn low(&self) -> f64 {
        self.low
    }

    #[getter]
    fn close(&self) -> f64 {
        self.close
    }

    #[getter]
    fn volume(&self) -> f64 {
        self.volume
    }

    #[getter]
    fn amount(&self) -> f64 {
        self.amount
    }

    fn __str__(&self) -> String {
        format!("{:#?}", self)
    }

    fn __repr__(&self) -> String {
        format!("{:#?}", self)
    }
}

#[derive(Debug, Deserialize)]
struct Size {
    size: f64,
    max: f64,
    min: f64,
}

#[derive(Debug, Deserialize)]
pub struct Product {
    symbol: String,
    delivery: Option<i64>,
    onboard: Option<i64>,
    order: HashSet<OrderType>,
    tif: Option<HashSet<Tif>>,
    price_filter: Size,
    lot_size: Size,
    min_notional: f64,
}

impl Product {
    pub fn symbol(&self) -> &String {
        &self.symbol
    }

    pub fn delivery(&self) -> DateTime<Tz> {
        match self.delivery {
            Some(delivery) => DateTime::from_timestamp_millis(delivery)
                .unwrap()
                .with_timezone(&Shanghai),
            None => DateTime::<Tz>::MAX_UTC.with_timezone(&Shanghai),
        }
    }

    pub fn onboard(&self) -> DateTime<Tz> {
        match self.onboard {
            Some(onboard) => DateTime::from_timestamp_millis(onboard)
                .unwrap()
                .with_timezone(&Shanghai),
            None => DateTime::<Tz>::MAX_UTC.with_timezone(&Shanghai),
        }
    }

    pub fn order_support(&self, order_type: &OrderType) -> bool {
        self.order.contains(order_type)
    }

    pub fn tif_support(&self, tif: &Tif) -> bool {
        match &self.tif {
            Some(inner) => inner.contains(tif),
            None => true,
        }
    }

    pub fn max_prc(&self) -> f64 {
        self.price_filter.max
    }

    pub fn min_prc(&self) -> f64 {
        self.price_filter.min
    }

    pub fn tick_size(&self) -> f64 {
        self.price_filter.size
    }

    pub fn lot(&self) -> f64 {
        self.lot_size.size
    }

    pub fn min_notional(&self) -> f64 {
        self.min_notional
    }
}

type Products = Response<Vec<Product>>;

#[derive(Debug, Deserialize)]
pub struct PositionRsp {
    pub session_id: u16,
    pub positions: Vec<Position>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Message {
    Success(Success),
    Login(LoginResponse),
    Error(ErrorResponse),
    Depth(Depth),
    Kline(Kline),
    Order(Order),
    Products(Products),
    Positions(Response<PositionRsp>),
    Position(Position),
    Close,
}

#[derive(Debug, Clone, Copy)]
#[pyclass]
pub enum EventType {
    Login,
    Depth,
    Kline,
    Order,
    Position,
}

#[derive(Debug)]
#[pyclass]
pub struct Event {
    event_type: EventType,
    data: Py<PyAny>,
}

impl Event {
    pub fn new<T: IntoPy<PyObject>>(event_type: EventType, data: T) -> Py<PyAny> {
        Python::with_gil(|py| {
            Self {
                event_type,
                data: data.into_py(py),
            }
            .into_py(py)
        })
    }
}
#[pymethods]
impl Event {
    #[getter]
    fn event_type(&self) -> EventType {
        self.event_type
    }

    #[getter]
    fn data(&self) -> &Py<PyAny> {
        &self.data
    }

    pub fn __repr__(&self) -> String {
        format!("{:?}", self)
    }

    pub fn __str__(&self) -> String {
        format!("{:?}", self)
    }
}

#[derive(Debug, Deserialize)]
#[pyclass]
pub struct Order {
    time: i64,
    symbol: String,
    side: Side,
    state: State,
    order_type: OrderType,
    tif: Tif,
    quantity: f64,
    price: f64,
    #[allow(unused)]
    order_id: i64,
    internal_id: u32,
    trade_time: i64,
    trade_price: f64,
    trade_quantity: f64,
    acc: f64,
    making: bool,
}

impl Order {
    pub fn new(
        id: u32,
        symbol: &str,
        price: f64,
        quantity: f64,
        side: Side,
        order_type: OrderType,
        tif: Tif,
    ) -> Self {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        Self {
            time: ts,
            price,
            quantity,
            symbol: symbol.into(),
            side,
            state: State::NEW,
            order_type,
            tif,
            order_id: -1,
            internal_id: id,
            trade_time: 0,
            trade_price: 0.0,
            trade_quantity: 0.0,
            acc: 0.0,
            making: false,
        }
    }

    pub fn on_update(&mut self, other: Self) {
        *self = other
    }
}
#[pymethods]
impl Order {
    #[getter]
    fn time(&self) -> i64 {
        self.time
    }
    #[getter]
    fn datetime(&self) -> DateTime<Tz> {
        DateTime::from_timestamp_millis(self.time as i64)
            .unwrap()
            .with_timezone(&Shanghai)
    }
    #[getter]
    fn symbol(&self) -> &str {
        &self.symbol
    }
    #[getter]
    fn side(&self) -> Side {
        self.side
    }
    #[getter]
    fn state(&self) -> State {
        self.state
    }
    #[getter]
    fn order_type(&self) -> OrderType {
        self.order_type
    }
    #[getter]
    fn tif(&self) -> Tif {
        self.tif
    }
    #[getter]
    fn quantity(&self) -> f64 {
        self.quantity
    }
    #[getter]
    fn price(&self) -> f64 {
        self.price
    }
    #[getter]
    pub fn id(&self) -> u32 {
        self.internal_id
    }
    #[getter]
    fn trade_time(&self) -> i64 {
        self.trade_time
    }
    #[getter]
    fn trade_dt(&self) -> DateTime<Tz> {
        DateTime::from_timestamp_millis(self.trade_time as i64)
            .unwrap()
            .with_timezone(&Shanghai)
    }
    #[getter]
    fn trade_price(&self) -> f64 {
        self.trade_price
    }
    #[getter]
    fn trade_quantity(&self) -> f64 {
        self.trade_quantity
    }
    #[getter]
    fn acc(&self) -> f64 {
        self.acc
    }
    #[getter]
    fn making(&self) -> bool {
        self.making
    }

    #[getter]
    pub fn is_active(&self) -> bool {
        matches!(self.state, State::NEW | State::PARTIALLY_FILLED)
    }

    fn __str__(&self) -> String {
        format!("{:#?}", self)
    }

    fn __repr__(&self) -> String {
        format!("{:#?}", self)
    }
}

#[derive(Debug, Serialize)]
pub struct OrderRequest {
    pub id: u32,
    pub symbol: String,
    pub price: f64,
    pub quantity: f64,
    pub side: Side,
    pub order_type: OrderType,
    pub tif: Tif,
    pub session_id: u16,
}

#[derive(Debug, Serialize)]
pub struct CancelRequest {
    pub symbol: String,
    pub session_id: u16,
    pub order_id: u32,
}

#[derive(Debug, Deserialize, Clone)]
#[pyclass]
pub struct Position {
    pub symbol: String,
    pub net: f64,
}

#[derive(Debug, Deserialize)]
#[pyclass]
#[allow(non_snake_case)]
pub struct PremiumIndex {
    #[serde(deserialize_with = "deserialize_symbol")]
    symbol: String,
    #[serde(deserialize_with = "string_to_f64")]
    markPrice: f64,
    #[serde(deserialize_with = "string_to_f64")]
    indexPrice: f64,
    #[serde(deserialize_with = "string_to_f64")]
    estimatedSettlePrice: f64,
    #[serde(deserialize_with = "string_to_f64")]
    lastFundingRate: f64,
    nextFundingTime: i64,
    #[serde(deserialize_with = "string_to_f64")]
    interestRate: f64,
    time: i64,
}

#[pymethods]
impl PremiumIndex {
    #[getter]
    fn time(&self) -> i64 {
        self.time
    }
    #[getter]
    fn datetime(&self) -> DateTime<Tz> {
        DateTime::from_timestamp_millis(self.time)
            .unwrap()
            .with_timezone(&Shanghai)
    }
    #[getter]
    fn symbol(&self) -> &str {
        &self.symbol
    }
    #[getter]
    fn mark_price(&self) -> f64 {
        self.markPrice
    }
    #[getter]
    fn index_price(&self) -> f64 {
        self.indexPrice
    }
    #[getter]
    fn estimated_settle_price(&self) -> f64 {
        self.estimatedSettlePrice
    }
    #[getter]
    fn last_funding_rate(&self) -> f64 {
        self.lastFundingRate
    }
    #[getter]
    fn next_funding_time(&self) -> i64 {
        self.nextFundingTime
    }
    #[getter]
    fn next_funding_dt(&self) -> DateTime<Tz> {
        DateTime::from_timestamp_millis(self.nextFundingTime as i64)
            .unwrap()
            .with_timezone(&Shanghai)
    }
    #[getter]
    fn interest_rate(&self) -> f64 {
        self.interestRate
    }
    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }

    fn __str__(&self) -> String {
        format!("{:?}", self)
    }
}

fn string_to_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    Ok(s.parse::<f64>().unwrap())
}

fn deserialize_symbol<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(String::deserialize(deserializer)?.to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order() {
        let s = r#"{"time":1716433595260,
                          "symbol":"dogeusdt",
                          "side":"SELL",
                          "state":"NEW",
                          "order_type":"LIMIT",
                          "tif":"GTC",
                          "quantity":25.0,
                          "price":0.2,
                          "order_id":45382967722,
                          "internal_id":2,
                          "trade_time":1716433595260,
                          "trade_price":0.0,
                          "trade_quantity":0.0,
                          "acc":0.0,
                          "making":false}"#;
        let o = serde_json::from_str::<Order>(s).unwrap();
        assert_eq!(o.time, 1716433595260);
        assert_eq!(o.side, Side::SELL);
        assert_eq!(o.state, State::NEW);
        assert_eq!(o.order_type, OrderType::LIMIT);
        assert_eq!(o.tif, Tif::GTC);
        assert_eq!(o.quantity, 25.0);
        assert_eq!(o.price, 0.2);
        assert_eq!(o.order_id, 45382967722);
        assert_eq!(o.internal_id, 2);
        assert_eq!(o.trade_time, 1716433595260);
        assert_eq!(o.trade_price, 0.0);
        assert_eq!(o.trade_quantity, 0.0);
        assert_eq!(o.acc, 0.0);
        assert!(!o.making);
    }
    #[test]
    fn test_depth() {
        let s = r#"{"time":1715093955172,
                          "symbol":"btcusdt",
                          "stream":"btcusdt@depth",
                          "bids":[{"price":63972.8,"quantity":0.0},
                                  {"price":63972.7,"quantity":0.0},
                                  {"price":63971.33,"quantity":0.0},
                                  {"price":63970.78,"quantity":0.0},
                                  {"price":63970.44,"quantity":0.0},
                                  {"price":63970.43,"quantity":0.0},
                                  {"price":63967.63,"quantity":0.0},
                                  {"price":63967.52,"quantity":0.0},
                                  {"price":63967.44,"quantity":0.0}],
                          "asks":[{"price":63962.83,"quantity":4.37492},
                                  {"price":63964.04,"quantity":0.0},
                                  {"price":63965.0,"quantity":0.0},
                                  {"price":63965.01,"quantity":0.0},
                                  {"price":63966.06,"quantity":0.0},
                                  {"price":63966.07,"quantity":0.0},
                                  {"price":63966.4,"quantity":0.0},
                                  {"price":63966.41,"quantity":0.0},
                                  {"price":63966.45,"quantity":0.00157},
                                  {"price":63966.52,"quantity":0.0}]}"#;
        let depth: Depth = serde_json::from_str(s).unwrap();
        assert_eq!(depth.stream(), "btcusdt@depth");
        assert_eq!(depth.symbol(), "btcusdt");
        assert_eq!(depth.time(), 1715093955172);
        assert_eq!(depth.bid_prc(0), 63972.8);
        assert_eq!(depth.bid_vol(0), 0.0);
        assert_eq!(depth.ask_prc(0), 63962.83);
        assert_eq!(depth.ask_vol(0), 4.37492);
        assert_eq!(depth.bid_level(), 9);
        assert_eq!(depth.ask_level(), 10);
        assert_eq!(depth.datetime().to_string(), "2024-05-07 22:59:15.172 CST");
    }

    #[test]
    fn test_kline() {
        let s = r#"{"time":1715098495999,
                          "symbol":"btcusdt",
                          "stream":"btcusdt@kline:1s",
                          "open":63763.07,
                          "high":63763.07,
                          "low":63763.07,
                          "close":63763.07,
                          "volume":0.03,
                          "amount":1912.8921}"#;

        let kline: Kline = serde_json::from_str(s).unwrap();
        assert_eq!(kline.symbol(), "btcusdt");
        assert_eq!(kline.stream(), "btcusdt@kline:1s");
        assert_eq!(kline.time(), 1715098495999);
        assert_eq!(kline.open(), 63763.07); // open, high, low, close, volume, amount
        assert_eq!(kline.high(), 63763.07);
        assert_eq!(kline.low(), 63763.07);
        assert_eq!(kline.close(), 63763.07);
        assert_eq!(kline.volume(), 0.03);
        assert_eq!(kline.amount(), 1912.8921);
        assert_eq!(kline.datetime().to_string(), "2024-05-08 00:14:55.999 CST");
    }

    #[test]
    fn test_product() {
        // spot
        let s = r#"[{"symbol":"reieth",
                           "delivery":null,
                           "onboard":null,
                           "order":["LIMIT","LIMIT_MAKER","MARKET","STOP_LOSS_LIMIT","TAKE_PROFIT_LIMIT"],
                           "tif":null,
                           "price_filter":{"size":1e-8,"max":1000.0,"min":1e-8},
                           "lot_size":{"size":0.1,"max":92141578.0,"min":0.1},
                           "market_lot_size":{"size":0.0,"max":358556.43736111,"min":0.0},
                           "min_notional":0.005},
                          {"symbol":"mantatry",
                           "delivery":null,
                           "onboard":null,
                           "order":["LIMIT","LIMIT_MAKER","MARKET","STOP_LOSS_LIMIT","TAKE_PROFIT_LIMIT"],
                           "tif":null,
                           "price_filter":{"size":0.01,"max":1000.0,"min":0.01},
                           "lot_size":{"size":0.1,"max":92141578.0,"min":0.1},
                           "market_lot_size":{"size":0.0,"max":5581.1376569,"min":0.0},
                           "min_notional":10.0}]"#;
        match serde_json::from_str::<Vec<Product>>(s) {
            Ok(p) => {
                println!("{:?}", p);
            }
            Err(e) => println!("{}", e),
        }
        // future
    }
}
