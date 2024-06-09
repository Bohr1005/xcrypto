use log::*;
use serde::Serialize;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::mpsc::UnboundedSender;
use xcrypto::chat::Side;
use xcrypto::position::PositionDB;
use xcrypto::{
    chat::{Position, State},
    tungstenite::Message,
};

use crate::OrderTrait;

pub struct Session {
    session_id: u16,
    positions: HashMap<String, Position>,
    posdb: Arc<PositionDB>,
    tx: Option<UnboundedSender<Message>>,
}

impl Session {
    pub async fn new(
        session_id: u16,
        posdb: Arc<PositionDB>,
        tx: UnboundedSender<Message>,
    ) -> anyhow::Result<Self> {
        let positions = posdb.get_positions(session_id);
        posdb.create_table(session_id).await?;
        Ok(Self {
            session_id,
            positions: positions.cloned().unwrap_or_default(),
            posdb,
            tx: Some(tx),
        })
    }

    pub fn active(&self) -> bool {
        self.tx.is_some()
    }

    pub fn on_order<T: OrderTrait + Serialize>(&mut self, order: &T) -> anyhow::Result<()> {
        if let State::FILLED | State::PARTIALLY_FILLED = order.state() {
            self.on_trade(order)?;
        }
        self.send(order)?;

        Ok(())
    }

    fn on_trade<T: OrderTrait>(&mut self, order: &T) -> anyhow::Result<()> {
        let mut binding = Position {
            symbol: order.symbol().into(),
            net: 0.0,
        };
        let position = self
            .positions
            .get_mut(order.symbol())
            .unwrap_or(&mut binding);

        let net = order.net()?;
        match order.side() {
            Side::BUY => position.net += net,
            Side::SELL => position.net -= net,
        }

        if let Some(position) = self.positions.get(order.symbol()) {
            self.send(position)?;
            self.posdb.update(self.session_id, position.to_owned())
        }

        Ok(())
    }

    fn send<T: Serialize>(&self, data: &T) -> anyhow::Result<()> {
        let msg = serde_json::to_string(data)?;
        if let Some(tx) = &self.tx {
            return Ok(tx.send(Message::Text(msg))?);
        }
        Ok(())
    }

    pub fn set_active(&mut self, tx: Option<UnboundedSender<Message>>) -> bool {
        info!(
            "Set session {} {} -> {}",
            self.session_id,
            self.active(),
            tx.is_some()
        );
        self.tx = tx;
        self.active()
    }
}
