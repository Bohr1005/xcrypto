pub mod chat;
pub mod constant;
pub mod phase;
pub mod rest;
pub mod session;
pub mod subscription;
pub mod ws;

use chat::*;
use constant::*;
use log::Record;
use logger::Level;
use phase::TradingPhase;
use pyo3::prelude::*;
use rest::*;
use session::*;
use std::str::FromStr;
use subscription::Subscription;

#[pyclass]
pub struct Handle {
    #[allow(unused)]
    inner: logger::Handle,
}

#[pyfunction]
fn init_logger(level: &str, path: Option<String>) -> Handle {
    let handle = logger::init(path, Level::from_str(level).unwrap());
    Handle { inner: handle }
}

#[pyfunction]
fn log_info(filename: String, lineno: u32, msg: String) {
    log::logger().log(
        &Record::builder()
            .args(format_args!("{}", msg))
            .level(log::Level::Info)
            .file(Some(filename.as_str()))
            .line(Some(lineno))
            .build(),
    );
}

#[pyfunction]
fn log_debug(filename: String, lineno: u32, msg: String) {
    log::logger().log(
        &Record::builder()
            .args(format_args!("{}", msg))
            .level(log::Level::Debug)
            .file(Some(filename.as_str()))
            .line(Some(lineno))
            .build(),
    );
}

#[pyfunction]
fn log_warn(filename: String, lineno: u32, msg: String) {
    log::logger().log(
        &Record::builder()
            .args(format_args!("{}", msg))
            .level(log::Level::Warn)
            .file(Some(filename.as_str()))
            .line(Some(lineno))
            .build(),
    );
}

#[pyfunction]
fn log_error(filename: String, lineno: u32, msg: String) {
    log::logger().log(
        &Record::builder()
            .args(format_args!("{}", msg))
            .level(log::Level::Error)
            .file(Some(filename.as_str()))
            .line(Some(lineno))
            .build(),
    );
}

#[pymodule]
fn pyalgo(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Kline>()?;
    m.add_class::<Depth>()?;
    m.add_class::<Order>()?;
    m.add_class::<Rest>()?;
    m.add_class::<Session>()?;
    m.add_class::<TradingPhase>()?;
    m.add_class::<Phase>()?;
    m.add_class::<Side>()?;
    m.add_class::<OrderType>()?;
    m.add_class::<Tif>()?;
    m.add_class::<State>()?;
    m.add_class::<EventType>()?;
    m.add_class::<Event>()?;
    m.add_class::<Subscription>()?;
    m.add_function(wrap_pyfunction!(init_logger, m)?)?;
    m.add_function(wrap_pyfunction!(log_info, m)?)?;
    m.add_function(wrap_pyfunction!(log_debug, m)?)?;
    m.add_function(wrap_pyfunction!(log_warn, m)?)?;
    m.add_function(wrap_pyfunction!(log_error, m)?)?;
    Ok(())
}
