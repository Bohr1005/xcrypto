use crate::chat::{CancelRequest, Message, OrderRequest, Product};
use crate::subscription::Subscription;
use crate::ws::WebSocketClient;
use crate::{constant::*, Order, PositionRsp};
use crate::{Event, Position};
use log::*;
use pyo3::prelude::*;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::time::{Duration, Instant};
use std::vec;
use xcrypto::chat::{Error, Login, LoginResponse, PositionReq, Request, Response};

#[pyclass]
pub struct Session {
    ws: WebSocketClient,
    session_id: u16,
    name: String,
    subscription: HashMap<String, Py<Subscription>>,
    orders: HashMap<u32, Py<Order>>,
    symbols: HashSet<String>,
    login: bool,
    trading: bool,
    id: u32,
    connection_time: Option<Instant>,
}

impl Session {
    fn login(&mut self) -> anyhow::Result<()> {
        self.send(
            "login",
            Login {
                session_id: self.session_id,
                name: self.name.clone(),
                trading: self.trading,
            },
        )?;
        Ok(())
    }

    fn on_login(&mut self, login: LoginResponse) -> Option<Py<PyAny>> {
        info!("{:?}", login);
        self.login = true;

        Some(Event::new(crate::EventType::Login, self.login))
    }

    fn get_products(&mut self) -> anyhow::Result<()> {
        self.send("get_products", Vec::<String>::new())?;
        Ok(())
    }

    fn get_positions(&mut self) -> anyhow::Result<()> {
        self.send(
            "get_positions",
            PositionReq {
                session_id: self.session_id,
                symbols: Vec::new(),
            },
        )?;
        Ok(())
    }

    fn on_products(&mut self, rsp: Response<Vec<Product>>) -> anyhow::Result<()> {
        let mut products = rsp.result;
        let cnt = products.len();
        while let Some(product) = products.pop() {
            info!("{:?}", product);
            let symbol = product.symbol().clone();
            let sub = Python::with_gil(|py| Py::new(py, Subscription::new(product)))?;
            self.subscription.insert(symbol, sub);
        }
        info!("Total products {}", cnt);
        self.get_positions()?;

        Ok(())
    }

    fn on_positions(&mut self, rsp: Response<PositionRsp>) -> anyhow::Result<()> {
        let result = rsp.result;
        let positions = result.positions;
        for position in positions {
            if let Some(sub) = self.subscription.get(&position.symbol) {
                Python::with_gil(|py| {
                    let mut pysub = sub.borrow_mut(py);
                    pysub.on_position(position)
                })
            }
        }

        info!("Session {} is ready", self.id);
        if !self.login {
            self.login()?;
        }

        Ok(())
    }

    fn on_error(&mut self, response: Response<Error>) {
        panic!("{:?}", response);
    }

    fn on_order(&mut self, order: Order) -> Option<Py<PyAny>> {
        info!("{:?}", order);
        let active = order.is_active();
        let id = order.id();
        match self.orders.get_mut(&id) {
            Some(pyorder) => {
                Python::with_gil(|py| {
                    let mut o = pyorder.borrow_mut(py);
                    o.on_update(order);
                });

                let e = Some(Event::new(crate::EventType::Order, pyorder.clone()));
                if !active {
                    self.orders.remove(&id);
                }
                return e;
            }
            None => warn!("Cannot find order, maybe a bug"),
        }
        None
    }

    fn on_position(&mut self, position: Position) {
        info!("{:?}", position);
        if let Some(sub) = self.subscription.get_mut(&position.symbol) {
            Python::with_gil(|py| {
                let mut sub = sub.borrow_mut(py);
                sub.on_position(position);
            })
        }
    }

    fn on_close(&mut self) {
        info!("Session {} is closed", self.id);
    }

    fn on_message(&mut self, msg: Message) -> Option<Py<PyAny>> {
        match msg {
            Message::Success(rsp) => info!(" {:?}", rsp),
            Message::Error(rsp) => self.on_error(rsp),
            Message::Login(rsp) => return self.on_login(rsp),
            Message::Products(rsp) => {
                if let Err(e) = self.on_products(rsp) {
                    error!("{}", e);
                }
            }
            Message::Positions(rsp) => {
                if let Err(e) = self.on_positions(rsp) {
                    error!("{}", e);
                }
            }
            Message::Kline(kline) => return Some(Event::new(crate::EventType::Kline, kline)),
            Message::Depth(depth) => return Some(Event::new(crate::EventType::Depth, depth)),
            Message::Order(order) => return self.on_order(order),
            Message::Position(position) => self.on_position(position),
            Message::Close => self.on_close(),
        }
        None
    }

    fn send<T: Debug + Serialize>(&mut self, method: &str, params: T) -> anyhow::Result<i64> {
        let id = self.id as i64;
        let req = Request {
            id: id,
            method: method.into(),
            params,
        };
        info!("{:?}", req);
        self.ws.send(req)?;
        self.id += 1;

        Ok(id)
    }
}

#[pymethods]
impl Session {
    #[new]
    fn new(addr: String, session_id: u16, name: String, trading: bool) -> Self {
        Self {
            ws: WebSocketClient::new(addr.clone()),
            session_id,
            name,
            subscription: HashMap::default(),
            orders: HashMap::default(),
            symbols: HashSet::default(),
            login: false,
            trading,
            id: 0,
            connection_time: None,
        }
    }

    #[getter]
    fn id(&self) -> u16 {
        self.session_id
    }

    #[getter]
    fn name(&self) -> &String {
        &self.name
    }

    #[getter]
    fn is_login(&self) -> bool {
        self.login
    }

    #[getter]
    fn trading(&self) -> bool {
        self.trading
    }

    fn connect(&mut self) {
        match self.connection_time {
            Some(t) => {
                if t.elapsed() > Duration::from_secs(30) {
                    self.connection_time.take();
                }
            }
            None => {
                self.ws.connect().unwrap();
                self.get_products().unwrap();
                self.connection_time = Some(Instant::now());

                while !self.login {
                    self.process();
                }

                self.ws.set_nonblocking(true).unwrap();
            }
        }
    }

    fn subscribe(&mut self, symbol: &str, stream: &str) -> PyResult<Py<Subscription>> {
        if !self.login {
            return Err(pyo3::exceptions::PyException::new_err("Please login first"));
        }

        let sub = self.subscription.get(symbol).cloned();
        match sub {
            Some(inner) => match self.send("subscribe", vec![format!("{}@{}", symbol, stream)]) {
                Ok(_) => {
                    self.symbols.insert(symbol.into());
                    return Ok(inner);
                }
                Err(e) => return Err(pyo3::exceptions::PyException::new_err(e.to_string())),
            },
            None => Err(pyo3::exceptions::PyException::new_err(format!(
                "Invalid symbol {}",
                symbol
            ))),
        }
    }

    fn add_order(
        &mut self,
        symbol: &str,
        price: f64,
        quantity: f64,
        side: &Side,
        order_type: &OrderType,
        tif: &Tif,
    ) -> Option<Py<Order>> {
        if !self.login || !self.trading {
            return None;
        }

        let id = self.id;
        let params = OrderRequest {
            id,
            symbol: symbol.into(),
            price,
            quantity,
            side: side.clone(),
            order_type: order_type.clone(),
            tif: tif.clone(),
            session_id: self.session_id,
        };

        if let Ok(_) = self.send("order", params) {
            let order = Order::new(
                id,
                symbol,
                price,
                quantity,
                side.to_owned(),
                order_type.to_owned(),
                tif.to_owned(),
            );

            let pyorder = Python::with_gil(|py| Py::new(py, order).unwrap());
            self.orders.insert(id, pyorder.clone());
            return Some(pyorder);
        }
        return None;
    }

    fn cancel(&mut self, symbol: String, order_id: u32) {
        if !self.login || !self.trading {
            return;
        }

        let params = CancelRequest {
            symbol,
            session_id: self.session_id,
            order_id,
        };

        if let Err(e) = self.send("cancel", params) {
            error!("{:?}", e);
        }
    }

    fn process(&mut self) -> Option<Py<PyAny>> {
        if let Some(msg) = self.ws.read() {
            debug!("{:?}", msg);
            return self.on_message(msg);
        }

        if self.ws.is_closed() {
            self.connect()
        }
        None
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        if let Err(e) = self.ws.close() {
            error!("{}", e);
        }
    }
}
