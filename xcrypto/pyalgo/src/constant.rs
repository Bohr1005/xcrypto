use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

#[pyclass]
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Phase {
    AUCTION,
    PRE_OPEN,
    OPEN,
    PRE_CLOSE,
    CLOSE,
    UNDEF,
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    BUY,
    SELL,
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Copy)]
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

#[pyclass]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Tif {
    GTC,
    IOC,
    FOK,
    GTX,
    GTD,
    UNDEF,
}

#[pyclass]
#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum State {
    NEW,
    #[allow(non_camel_case_types)]
    PARTIALLY_FILLED,
    FILLED,
    CANCELED,
    REJECTED,
    EXPIRED,
    #[allow(non_camel_case_types)]
    EXPIRED_IN_MATCH,
    UNDEF,
}
