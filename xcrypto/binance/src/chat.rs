#![allow(non_snake_case)]
use native_json::json;
use serde::{
    de::{SeqAccess, Visitor},
    ser::SerializeStruct,
    Deserialize, Deserializer, Serialize, Serializer,
};
use xcrypto::chat::*;

use crate::{ListenKey, OrderTrait};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct KlineData {
    pub t: i64,
    pub T: i64,
    pub i: String,
    pub f: i64,
    pub L: i64,
    pub o: String,
    pub c: String,
    pub h: String,
    pub l: String,
    pub v: String,
    pub x: bool,
    pub q: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(into = "Kline")]
pub struct BinanceKline {
    #[serde(deserialize_with = "deserialize_symbol")]
    pub s: String,
    pub k: KlineData,
}

impl BinanceKline {
    pub fn stream(&self) -> String {
        format!("{}@kline:{}", self.s, self.k.i)
    }
}

impl From<BinanceKline> for Kline {
    fn from(value: BinanceKline) -> Self {
        Kline {
            time: value.k.T,
            symbol: value.s.clone(),
            stream: format!("{}@kline:{}", value.s, value.k.i),
            open: value.k.o.parse().unwrap_or_default(),
            high: value.k.h.parse().unwrap_or_default(),
            low: value.k.l.parse().unwrap_or_default(),
            close: value.k.c.parse().unwrap_or_default(),
            volume: value.k.v.parse().unwrap_or_default(),
            amount: value.k.q.parse().unwrap_or_default(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct BinanceQuote {
    price: f64,
    quantity: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(into = "Depth<BinanceQuote>")]
pub struct BinanceDepth {
    pub E: i64,
    #[serde(deserialize_with = "deserialize_symbol")]
    s: String,
    b: Vec<BinanceQuote>,
    a: Vec<BinanceQuote>,
}

impl BinanceDepth {
    pub fn stream(&self) -> String {
        format!("{}@depth", self.s)
    }

    pub fn reverse(&mut self) {
        self.b.reverse();
    }

    pub fn bid(&self, level: usize) -> f64 {
        match self.b.get(level) {
            Some(bid) => bid.price,
            None => 0.0,
        }
    }
    pub fn ask(&self, level: usize) -> f64 {
        match self.a.get(level) {
            Some(ask) => ask.price,
            None => 0.0,
        }
    }
}

impl<'de> Deserialize<'de> for BinanceQuote {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct QuoteVisitor;
        impl<'de> Visitor<'de> for QuoteVisitor {
            type Value = BinanceQuote;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a sequence of two elements [price, quantity]")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let price = seq
                    .next_element::<String>()?
                    .ok_or_else(|| serde::de::Error::missing_field("missing quantity"))?
                    .parse::<f64>()
                    .expect("parse price failed");

                let quantity = seq
                    .next_element::<String>()?
                    .ok_or_else(|| serde::de::Error::missing_field("missing quantity"))?
                    .parse::<f64>()
                    .expect("parse quantity failed");

                Ok(BinanceQuote { price, quantity })
            }
        }

        deserializer.deserialize_seq(QuoteVisitor)
    }
}

impl From<BinanceDepth> for Depth<BinanceQuote> {
    fn from(value: BinanceDepth) -> Self {
        Depth {
            time: value.E,
            symbol: value.s.clone(),
            stream: format!("{}@depth", value.s),
            bids: value.b,
            asks: value.a,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "filterType")]
#[allow(non_camel_case_types, unused)]
pub enum FilterField {
    PRICE_FILTER {
        tickSize: String,
        maxPrice: String,
        minPrice: String,
    },
    LOT_SIZE {
        stepSize: String,
        maxQty: String,
        minQty: String,
    },
    MARKET_LOT_SIZE {
        stepSize: String,
        maxQty: String,
        minQty: String,
    },
    MAX_NUM_ORDERS {
        maxNumOrders: Option<i64>,
        limit: Option<i64>,
    },
    MAX_NUM_ALGO_ORDERS {
        maxNumAlgoOrders: Option<i64>,
        limit: Option<i64>,
    },
    MAX_NUM_ICEBERG_ORDERS {
        maxNumIcebergOrders: i64,
    },
    MIN_NOTIONAL {
        notional: String,
    },
    MAX_POSITION {
        maxPosition: String,
    },
    NOTIONAL {
        minNotional: String,
        applyMinToMarket: bool,
        maxNotional: String,
        applyMaxToMarket: bool,
        avgPriceMins: i32,
    },
    PERCENT_PRICE {
        multiplierDecimal: String,
        multiplierDown: String,
        multiplierUp: String,
    },
    ICEBERG_PARTS {
        limit: i32,
    },
    TRAILING_DELTA {
        minTrailingAboveDelta: i32,
        maxTrailingAboveDelta: i32,
        minTrailingBelowDelta: i32,
        maxTrailingBelowDelta: i32,
    },
    PERCENT_PRICE_BY_SIDE {
        bidMultiplierUp: String,
        bidMultiplierDown: String,
        askMultiplierUp: String,
        askMultiplierDown: String,
        avgPriceMins: i32,
    },
    EXCHANGE_MAX_NUM_ORDERS {
        maxNumOrders: i64,
    },
    EXCHANGE_MAX_ALGO_ORDERS {
        maxNumAlgoOrders: i64,
    },
    EXCHANGE_MAX_NUM_ICEBERG_ORDERS {
        maxNumIcebergOrders: i64,
    },
}

#[allow(non_camel_case_types)]
#[derive(Debug, Deserialize, Clone, PartialEq)]
pub enum ConctactStatus {
    PENDING_TRADING,
    TRADING,
    PRE_DELIVERING,
    DELIVERING,
    DELIVERED,
    PRE_SETTLE,
    SETTLING,
    CLOSE,
    PRE_TRADING,
    POST_TRADING,
    END_OF_DAY,
    HALT,
    BREAK,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BinanceProduct {
    #[serde(deserialize_with = "deserialize_symbol")]
    pub symbol: String,
    pub status: ConctactStatus,
    #[serde(default)]
    pub deliveryDate: Option<u64>,
    #[serde(default)]
    pub onboardDate: Option<u64>,
    pub filters: Vec<FilterField>,
    pub orderTypes: Vec<String>,
    pub timeInForce: Option<Vec<String>>,
}

impl Serialize for BinanceProduct {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct Size {
            size: f64,
            max: f64,
            min: f64,
        }

        let mut state = serializer.serialize_struct("BinanceProduct", 6)?;
        state.serialize_field("symbol", &self.symbol.to_lowercase())?;
        state.serialize_field("delivery", &self.deliveryDate)?;
        state.serialize_field("onboard", &self.onboardDate)?;
        state.serialize_field("order", &self.orderTypes)?;
        state.serialize_field("tif", &self.timeInForce)?;

        for filter in self.filters.iter() {
            match filter {
                FilterField::PRICE_FILTER {
                    tickSize,
                    maxPrice,
                    minPrice,
                } => {
                    state.serialize_field(
                        "price_filter",
                        &Size {
                            size: tickSize.parse::<f64>().unwrap(),
                            max: maxPrice.parse::<f64>().unwrap(),
                            min: minPrice.parse::<f64>().unwrap(),
                        },
                    )?;
                }
                FilterField::LOT_SIZE {
                    stepSize,
                    maxQty,
                    minQty,
                } => {
                    state.serialize_field(
                        "lot_size",
                        &Size {
                            size: stepSize.parse::<f64>().unwrap(),
                            max: maxQty.parse::<f64>().unwrap(),
                            min: minQty.parse::<f64>().unwrap(),
                        },
                    )?;
                }
                FilterField::NOTIONAL { minNotional, .. } => {
                    state.serialize_field("min_notional", &minNotional.parse::<f64>().unwrap())?;
                }
                FilterField::MIN_NOTIONAL { notional } => {
                    state.serialize_field("min_notional", &notional.parse::<f64>().unwrap())?;
                }
                _ => (),
            }
        }
        state.end()
    }
}

json! {
    ProductResponse {
        symbols: [BinanceProduct]
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BinanceOrder {
    pub id: u32,
    pub symbol: String,
    pub price: f64,
    pub quantity: f64,
    pub side: Side,
    pub order_type: OrderType,
    pub tif: Tif,
    pub session_id: u16,
}

#[derive(Debug, Deserialize)]
pub struct BinanceCancel {
    pub symbol: String,
    pub session_id: u16,
    pub order_id: u32,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum MarketStream {
    Depth(BinanceDepth),
    Kline(BinanceKline),
}

// spot
json! {
    SpotListenKey {
        listenKey: String
    }
}

impl ListenKey for SpotListenKey {
    fn key(&self) -> &str {
        self.listenKey.as_str()
    }
}

json! {
    SpotPosition {
        a: String,
        f: String,
        l: String
    }
}

json! {
    OutboundAccountPosition {
        e: String,
        E: i64,
        u: i64,
        B: [SpotPosition]
    }
}

json! {
    BalanceUpdate {
        e: String,
        E: i64,
        a: String,
        d: String,
        T: i64
    }
}

json! {
    SpotExpired {
        e: String,
        E: String,
        listenKey: String
    }
}

json! {
    UserLiabilityChange {
        e: String,
        E: i64,
        a: String,
        t: String,
        p: String,
        i: String,
    }
}

json! {
    MarginLevelStatusChange {
        e: String,
        E: i64,
        l: String,
        s: String
    }
}

json! {
    OCODetails{
        s: String,
        i: i64,
        c: String
    }
}

json! {
    ListenStatus {
        e: String,
        E: i64,
        s: String,
        g: i64,
        o: String,
        l: String,
        L: String,
        r: String,
        C: String,
        T: i64,
        O: [OCODetails]
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(into = "Order")]
pub struct ExecutionReport {
    pub E: i64,
    #[serde(deserialize_with = "deserialize_symbol")]
    pub s: String,
    pub S: Side,
    pub o: String,
    pub f: String,
    pub q: String,
    pub p: String,
    pub c: String,
    pub C: String,
    pub X: State,
    pub i: i64,
    pub N: Option<String>,
    #[serde(deserialize_with = "deserialize_somef64")]
    pub n: Option<f64>,
    pub T: i64,
    pub t: i64,
    pub l: String,
    pub L: String,
    pub z: String,
    pub m: bool,
    pub O: i64,
}

impl OrderTrait for ExecutionReport {
    fn commission(&self) -> f64 {
        self.n.unwrap_or(0.0)
    }
    fn net(&self) -> anyhow::Result<f64> {
        Ok(self.trd_vol()? - self.commission())
    }
    fn side(&self) -> Side {
        self.S
    }
    fn state(&self) -> State {
        self.X
    }
    fn symbol(&self) -> &str {
        self.s.as_str()
    }
    fn trd_vol(&self) -> anyhow::Result<f64> {
        Ok(self.l.parse::<f64>()?)
    }
}

impl From<ExecutionReport> for Order {
    fn from(value: ExecutionReport) -> Self {
        let client_order_id = match value.X {
            State::CANCELED => &value.C,
            _ => &value.c,
        }
        .parse::<u64>()
        .unwrap_or_default();

        let internal_id = client_order_id & 0xFFFFFFFF;
        Self {
            time: value.E,
            symbol: value.s,
            side: value.S,
            state: value.X,
            order_type: value.o.parse().unwrap(),
            tif: value.f.parse().unwrap(),
            quantity: value.q.parse().unwrap_or_default(),
            price: value.p.parse().unwrap_or_default(),
            order_id: value.i,
            internal_id: internal_id as u32,
            trade_time: value.T,
            trade_price: value.L.parse().unwrap_or_default(),
            trade_quantity: value.l.parse().unwrap_or_default(),
            acc: value.z.parse().unwrap_or_default(),
            making: value.m,
        }
    }
}

// usdt

json! {
    UsdtListenKey {
        listenKey: String
    }
}

impl ListenKey for UsdtListenKey {
    fn key(&self) -> &str {
        self.listenKey.as_str()
    }
}

json! {
    UsdtExpired {
        stream: String,
        data: {
            e: String,
            E: String,
            listenKey: String,
        }
    }
}

json! {
    MarginItem {
        s: String,
        ps: String,
        pa: String,
        mt: String,
        iw: String,
        mp: String,
        up: String,
        mm: String,
    }
}

json! {
    MarginCall {
        E: i64,
        cw: String,
        p: [MarginItem]
    }
}

json! {
    UsdtPosition {
        s: String,
        pa: String,
        ep: String,
        bep: String,
        cr: String,
        up: String,
        mt: String,
        iw: String,
        ps: String
    }
}

json! {
    Asset {
        a: String,
        wb: String,
        cw: String,
        bc: String
    }
}

json! {
    AccountUpdate {
        E: i64,
        T: i64,
        a: {
            m: String,
            B: [Asset],
            P: [Position]
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]

pub struct OrderData {
    #[serde(deserialize_with = "deserialize_symbol")]
    pub s: String,
    pub c: String,
    pub S: Side,
    pub o: String,
    pub f: String,
    pub q: String,
    pub p: String,
    pub X: State,
    pub i: i64,
    pub N: Option<String>,
    #[serde(deserialize_with = "deserialize_somef64")]
    pub n: Option<f64>,
    pub T: i64,
    pub t: i64,
    pub L: String,
    pub l: String,
    pub z: String,
    pub m: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(into = "Order")]
pub struct OrderUpdate {
    pub E: i64,
    pub o: OrderData,
}

impl OrderTrait for OrderUpdate {
    fn commission(&self) -> f64 {
        self.o.n.unwrap_or(0.0)
    }
    fn net(&self) -> anyhow::Result<f64> {
        self.trd_vol()
    }
    fn side(&self) -> Side {
        self.o.S
    }
    fn state(&self) -> State {
        self.o.X
    }
    fn symbol(&self) -> &str {
        self.o.s.as_str()
    }
    fn trd_vol(&self) -> anyhow::Result<f64> {
        Ok(self.o.l.parse::<f64>()?)
    }
}
impl From<OrderUpdate> for Order {
    fn from(value: OrderUpdate) -> Self {
        let o = value.o;
        let client_order_id = o.c.parse::<u64>().unwrap_or_default();
        let internal_id = client_order_id & 0xFFFFFFFF;

        Self {
            time: value.E,
            symbol: o.s,
            side: o.S,
            state: o.X,
            order_type: o.o.parse().unwrap(),
            tif: o.f.parse().unwrap(),
            quantity: o.q.parse().unwrap_or_default(),
            price: o.p.parse().unwrap_or_default(),
            order_id: o.i,
            internal_id: internal_id as u32,
            trade_time: o.T,
            trade_price: o.L.parse().unwrap_or_default(),
            trade_quantity: o.l.parse().unwrap_or_default(),
            acc: o.z.parse().unwrap_or_default(),
            making: o.m,
        }
    }
}

json! {
    AccountConfigUpdate {
        E: i64,
        T: i64,
        ac: {
            s: String,
            l: u16
        }
    }
}

json! {
    MultiAssetsAccountConfigUpdate {
        E: i64,
        T: i64,
        ai: {
            j: bool
        }
    }
}

json! {
    StrategyUpdate {
        T: i64,
        E: i64,
        su: {
            si: i64,
            st: String,
            ss: String,
            s: String,
            ut: i64,
            c: i32,
        }
    }
}

json! {
    GridUpdate {
        T: i64,
        E: i64,
        gu: {
            si: i64,
            st: String,
            ss: String,
            s: String,
            r: String,
            up: String,
            uq: String,
            uf: String,
            mp: String,
            ut: i64
        }
    }
}

json! {
    ConditionalOrderTriggerReject {
        E: i64,
        T: i64,
        or: {
            s: String,
            i: i64,
            r: String,
        }
    }
}

json! {
    RiskLevelChange {
        E: i64,
        u: String,
        s: String,
        eq: String,
        ae: String,
        m: String
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
#[allow(unused)]
pub enum Event {
    Success(Response<Option<i64>>),
    Error(ErrorResponse),
    Stream(MarketStream),
    // spot
    ExecutionReport(ExecutionReport),
    SpotListenKey(SpotListenKey),
    Balance(BalanceUpdate),
    OutboundAccountPosition(OutboundAccountPosition),
    SpotExpired(SpotExpired),
    ListenStatus(ListenStatus),
    UserLiabilityChange(UserLiabilityChange),
    MarginLevelStatusChange(MarginLevelStatusChange),
    // usdt
    OrderUpdate(OrderUpdate),
    UsdtListenKey(UsdtListenKey),
    UsdtExpired(UsdtExpired),
    MarginCall(MarginCall),
    UsdtPosition(UsdtPosition),
    AccountUpdate(AccountUpdate),
    AccountConfigUpdate(AccountConfigUpdate),
    MultiAssetsAccountConfigUpdate(MultiAssetsAccountConfigUpdate),
    StrategyUpdate(StrategyUpdate),
    GridUpdate(GridUpdate),
    ConditionalOrderTriggerReject(ConditionalOrderTriggerReject),
    RiskLevelChange(RiskLevelChange),
}

fn deserialize_symbol<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(String::deserialize(deserializer)?.to_lowercase())
}

fn deserialize_somef64<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    match Option::<String>::deserialize(deserializer)? {
        Some(inner) => Ok(Some(inner.parse().unwrap_or_default())),
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_depth() {
        let s = r#"{"e":"depthUpdate",
                          "E":1714977197753,
                          "s":"BTCUSDT",
                          "U":46732844503,
                          "u":46732844710,
                          "b":[["64280.00000000","1.51596000"],
                               ["64279.81000000","0.18344000"],
                               ["64278.01000000","0.01907000"],
                               ["64277.37000000","0.00312000"],
                               ["64276.96000000","0.00000000"]],
                          "a":[["64280.01000000","7.16998000"],
                               ["64280.13000000","0.00000000"],
                               ["64280.74000000","0.00000000"],
                               ["64280.98000000","0.00155000"],
                               ["64281.46000000","0.00000000"]]}"#;
        let binancedepth: BinanceDepth = serde_json::from_str(s).unwrap();
        let depth: Depth<BinanceQuote> = binancedepth.into();
        assert_eq!(depth.time, 1714977197753);
        assert_eq!(depth.symbol, "btcusdt");
        assert_eq!(depth.stream, "btcusdt@depth");
        assert_eq!(depth.bids.len(), 5);
        assert_eq!(depth.asks.len(), 5);
    }

    #[test]
    fn test_kline() {
        let s = r#"{
            "e": "kline",     
            "E": 123456789,   
            "s": "BNBUSDT",   
            "k": {
              "t": 123400000, 
              "T": 123460000, 
              "s": "BNBUSDT", 
              "i": "1m",      
              "f": 100,       
              "L": 200,       
              "o": "0.0010",  
              "c": "0.0020",  
              "h": "0.0025",  
              "l": "0.0015",  
              "v": "1000",    
              "n": 100,       
              "x": false,     
              "q": "1.0000",  
              "V": "500",     
              "Q": "0.500",   
              "B": "123456"   
            }
          }"#;
        let binancekline: BinanceKline = serde_json::from_str(s).unwrap();
        let kline: Kline = binancekline.into();

        assert_eq!(kline.time, 123460000);
        assert_eq!(kline.symbol, "bnbusdt");
        assert_eq!(kline.stream, "bnbusdt@kline:1m");
        assert_eq!(kline.open, 0.001);
        assert_eq!(kline.high, 0.0025);
        assert_eq!(kline.low, 0.0015);
        assert_eq!(kline.close, 0.002);
        assert_eq!(kline.volume, 1000.0);
        assert_eq!(kline.amount, 1.0000);
    }

    #[test]
    fn test_product() {
        // spot
        let s = r#"{"symbol": "ETHBTC", 
                          "status": "TRADING", 
                          "baseAsset": "ETH", 
                          "baseAssetPrecision": 8, 
                          "quoteAsset": "BTC", 
                          "quotePrecision": 8, 
                          "quoteAssetPrecision": 8, 
                          "baseCommissionPrecision": 8, 
                          "quoteCommissionPrecision": 8, 
                          "orderTypes": ["LIMIT", "LIMIT_MAKER", "MARKET", "STOP_LOSS_LIMIT", "TAKE_PROFIT_LIMIT"], 
                          "icebergAllowed": true, 
                          "ocoAllowed": true, 
                          "otoAllowed": false, 
                          "quoteOrderQtyMarketAllowed": true, 
                          "allowTrailingStop": true, 
                          "cancelReplaceAllowed": true, 
                          "isSpotTradingAllowed": true, 
                          "isMarginTradingAllowed": true, 
                          "filters": [{"filterType": "PRICE_FILTER", "minPrice": "0.00001000", "maxPrice": "922327.00000000", "tickSize": "0.00001000"}, 
                                      {"filterType": "LOT_SIZE", "minQty": "0.00010000", "maxQty": "100000.00000000", "stepSize": "0.00010000"}, 
                                      {"filterType": "ICEBERG_PARTS", "limit": 10}, 
                                      {"filterType": "MARKET_LOT_SIZE", "minQty": "0.00000000", "maxQty": "2703.20648368", "stepSize": "0.00000000"}, 
                                      {"filterType": "TRAILING_DELTA", "minTrailingAboveDelta": 10, "maxTrailingAboveDelta": 2000, "minTrailingBelowDelta": 10, "maxTrailingBelowDelta": 2000}, 
                                      {"filterType": "PERCENT_PRICE_BY_SIDE", "bidMultiplierUp": "5", "bidMultiplierDown": "0.2", "askMultiplierUp": "5", "askMultiplierDown": "0.2", "avgPriceMins": 5}, 
                                      {"filterType": "NOTIONAL", "minNotional": "0.00010000", "applyMinToMarket": true, "maxNotional": "9000000.00000000", "applyMaxToMarket": false, "avgPriceMins": 5}, 
                                      {"filterType": "MAX_NUM_ORDERS", "maxNumOrders": 200}, 
                                      {"filterType": "MAX_NUM_ALGO_ORDERS", "maxNumAlgoOrders": 5}], 
                            "permissions": [], 
                            "permissionSets": [["SPOT", "MARGIN", "TRD_GRP_004", "TRD_GRP_005", "TRD_GRP_006", "TRD_GRP_008", "TRD_GRP_009", "TRD_GRP_010", "TRD_GRP_011", "TRD_GRP_012", "TRD_GRP_013", "TRD_GRP_014", "TRD_GRP_015", "TRD_GRP_016", "TRD_GRP_017", "TRD_GRP_018", "TRD_GRP_019", "TRD_GRP_020", "TRD_GRP_021", "TRD_GRP_022", "TRD_GRP_023", "TRD_GRP_024", "TRD_GRP_025"]], 
                            "defaultSelfTradePreventionMode": "EXPIRE_MAKER", 
                            "allowedSelfTradePreventionModes": ["EXPIRE_TAKER", "EXPIRE_MAKER", "EXPIRE_BOTH"]}"#;
        let product: BinanceProduct = serde_json::from_str(s).unwrap();
        assert_eq!(product.symbol, "ethbtc");
        assert_eq!(product.status, ConctactStatus::TRADING);
        assert_eq!(product.deliveryDate, None);
        assert_eq!(product.onboardDate, None);
        assert_eq!(product.filters.len(), 9);
        assert_eq!(product.orderTypes.len(), 5);
        assert_eq!(product.timeInForce, None);

        // future
        let s = r#"{"symbol": "BTCUSDT", 
                          "pair": "BTCUSDT", 
                          "contractType": "PERPETUAL", 
                          "deliveryDate": 4133404800000, 
                          "onboardDate": 1569398400000, 
                          "status": "TRADING", 
                          "maintMarginPercent": "2.5000", 
                          "requiredMarginPercent": "5.0000", 
                          "baseAsset": "BTC", 
                          "quoteAsset": "USDT", 
                          "marginAsset": "USDT", 
                          "pricePrecision": 2, 
                          "quantityPrecision": 3, 
                          "baseAssetPrecision": 8, 
                          "quotePrecision": 8, 
                          "underlyingType": "COIN", 
                          "underlyingSubType": ["PoW"], 
                          "settlePlan": 0, 
                          "triggerProtect": "0.0500", 
                          "liquidationFee": "0.012500", 
                          "marketTakeBound": "0.05", 
                          "maxMoveOrderLimit": 10000, 
                          "filters": [{"maxPrice": "4529764", "minPrice": "556.80", "filterType": "PRICE_FILTER", "tickSize": "0.10"}, 
                                      {"stepSize": "0.001", "maxQty": "1000", "filterType": "LOT_SIZE", "minQty": "0.001"}, 
                                      {"minQty": "0.001", "stepSize": "0.001", "filterType": "MARKET_LOT_SIZE", "maxQty": "120"}, 
                                      {"limit": 200, "filterType": "MAX_NUM_ORDERS"}, 
                                      {"filterType": "MAX_NUM_ALGO_ORDERS", "limit": 10}, 
                                      {"notional": "100", "filterType": "MIN_NOTIONAL"}, 
                                      {"multiplierDecimal": "4", "filterType": "PERCENT_PRICE", "multiplierDown": "0.9500", "multiplierUp": "1.0500"}], 
                          "orderTypes": ["LIMIT", "MARKET", "STOP", "STOP_MARKET", "TAKE_PROFIT", "TAKE_PROFIT_MARKET", "TRAILING_STOP_MARKET"], 
                          "timeInForce": ["GTC", "IOC", "FOK", "GTX", "GTD"]}"#;
        let product: BinanceProduct = serde_json::from_str(s).unwrap();

        assert_eq!(product.symbol, "btcusdt");
        assert_eq!(product.status, ConctactStatus::TRADING);
        assert_eq!(product.deliveryDate, Some(4133404800000));
        assert_eq!(product.onboardDate, Some(1569398400000));
        assert_eq!(product.filters.len(), 7);
        assert_eq!(product.orderTypes.len(), 7);
        assert_eq!(product.timeInForce.unwrap().len(), 5);
    }

    #[test]
    fn test_product_response() {
        let s = r#"{"timezone": "UTC", 
                          "serverTime": 1715054406944, 
                          "futuresType": "U_MARGINED", 
                          "rateLimits": [{"rateLimitType": "REQUEST_WEIGHT", "interval": "MINUTE", "intervalNum": 1, "limit": 2400}, 
                                         {"rateLimitType": "ORDERS", "interval": "MINUTE", "intervalNum": 1, "limit": 1200}, 
                                         {"rateLimitType": "ORDERS", "interval": "SECOND", "intervalNum": 10, "limit": 300}], 
                          "exchangeFilters": [], 
                          "assets": [{"asset": "USDT", "marginAvailable": true, "autoAssetExchange": "-10000"}, 
                                     {"asset": "BTC", "marginAvailable": true, "autoAssetExchange": "-0.10000000"}, 
                                     {"asset": "BNB", "marginAvailable": true, "autoAssetExchange": "-10"}, 
                                     {"asset": "ETH", "marginAvailable": true, "autoAssetExchange": "-5"}, 
                                     {"asset": "XRP", "marginAvailable": true, "autoAssetExchange": "0"},
                                     {"asset": "USDC", "marginAvailable": true, "autoAssetExchange": "-10000"}, 
                                     {"asset": "TUSD", "marginAvailable": true, "autoAssetExchange": "0"}, 
                                     {"asset": "FDUSD", "marginAvailable": true, "autoAssetExchange": "0"}], 
                          "symbols": [{"symbol": "BTCUSDT", 
                                       "pair": "BTCUSDT", 
                                       "contractType": "PERPETUAL", 
                                       "deliveryDate": 4133404800000, 
                                       "onboardDate": 1569398400000, 
                                       "status": "TRADING", 
                                       "maintMarginPercent": "2.5000", 
                                       "requiredMarginPercent": "5.0000", 
                                       "baseAsset": "BTC", 
                                       "quoteAsset": "USDT", 
                                       "marginAsset": "USDT", 
                                       "pricePrecision": 2, 
                                       "quantityPrecision": 3, 
                                       "baseAssetPrecision": 8, 
                                       "quotePrecision": 8, 
                                       "underlyingType": "COIN", 
                                       "underlyingSubType": ["PoW"], 
                                       "settlePlan": 0, 
                                       "triggerProtect": "0.0500", 
                                       "liquidationFee": "0.012500", 
                                       "marketTakeBound": "0.05", 
                                       "maxMoveOrderLimit": 10000, 
                                       "filters": [{"tickSize": "0.10", "maxPrice": "4529764", "filterType": "PRICE_FILTER", "minPrice": "556.80"}, 
                                                   {"minQty": "0.001", "stepSize": "0.001", "filterType": "LOT_SIZE", "maxQty": "1000"}, 
                                                   {"minQty": "0.001", "filterType": "MARKET_LOT_SIZE", "maxQty": "120", "stepSize": "0.001"}, 
                                                   {"filterType": "MAX_NUM_ORDERS", "limit": 200}, 
                                                   {"limit": 10, "filterType": "MAX_NUM_ALGO_ORDERS"}, 
                                                   {"notional": "100", "filterType": "MIN_NOTIONAL"}, 
                                                   {"multiplierDecimal": "4", "multiplierUp": "1.0500", "multiplierDown": "0.9500", "filterType": "PERCENT_PRICE"}], 
                                       "orderTypes": ["LIMIT", "MARKET", "STOP", "STOP_MARKET", "TAKE_PROFIT", "TAKE_PROFIT_MARKET", "TRAILING_STOP_MARKET"], 
                                       "timeInForce": ["GTC", "IOC", "FOK", "GTX", "GTD"]}, 
                                      {"symbol": "ETHUSDT", 
                                       "pair": "ETHUSDT", 
                                       "contractType": "PERPETUAL", 
                                       "deliveryDate": 4133404800000, 
                                       "onboardDate": 1569398400000, 
                                       "status": "TRADING", 
                                       "maintMarginPercent": "2.5000", 
                                       "requiredMarginPercent": "5.0000", 
                                       "baseAsset": "ETH", 
                                       "quoteAsset": "USDT", 
                                       "marginAsset": "USDT", 
                                       "pricePrecision": 2, 
                                       "quantityPrecision": 3, 
                                       "baseAssetPrecision": 8, 
                                       "quotePrecision": 8, 
                                       "underlyingType": "COIN", 
                                       "underlyingSubType": ["Layer-1"], 
                                       "settlePlan": 0, 
                                       "triggerProtect": "0.0500", 
                                       "liquidationFee": "0.012500", 
                                       "marketTakeBound": "0.05", 
                                       "maxMoveOrderLimit": 10000, 
                                       "filters": [{"filterType": "PRICE_FILTER", "minPrice": "39.86", "tickSize": "0.01", "maxPrice": "306177"}, 
                                                   {"maxQty": "10000", "stepSize": "0.001", "filterType": "LOT_SIZE", "minQty": "0.001"}, 
                                                   {"filterType": "MARKET_LOT_SIZE", "minQty": "0.001", "stepSize": "0.001", "maxQty": "2000"}, 
                                                   {"filterType": "MAX_NUM_ORDERS", "limit": 200}, 
                                                   {"filterType": "MAX_NUM_ALGO_ORDERS", "limit": 10}, 
                                                   {"filterType": "MIN_NOTIONAL", "notional": "20"}, 
                                                   {"multiplierUp": "1.0500", "multiplierDown": "0.9500", "filterType": "PERCENT_PRICE", "multiplierDecimal": "4"}], 
                                       "orderTypes": ["LIMIT", "MARKET", "STOP", "STOP_MARKET", "TAKE_PROFIT", "TAKE_PROFIT_MARKET", "TRAILING_STOP_MARKET"], 
                                       "timeInForce": ["GTC", "IOC", "FOK", "GTX", "GTD"]}]}"#;
        let rsp: ProductResponse = serde_json::from_str(&s).unwrap();
        println!("{:?}", rsp);
    }
}
