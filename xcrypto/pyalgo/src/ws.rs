use crate::chat;
use log::{debug, error, info};
use serde::Serialize;
use std::fmt::Debug;
use std::net::TcpStream;
use websocket::client::sync::Client;
use websocket::client::ClientBuilder;
use websocket::OwnedMessage;

pub struct WebSocketClient {
    addr: String,
    inner: Option<Client<TcpStream>>,
}

impl WebSocketClient {
    pub fn new(addr: String) -> Self {
        WebSocketClient { addr, inner: None }
    }

    pub fn is_closed(&self) -> bool {
        self.inner.is_none()
    }

    pub fn connect(&mut self) -> anyhow::Result<()> {
        let client = ClientBuilder::new(&self.addr)?.connect_insecure()?;
        self.inner.replace(client);

        Ok(())
    }

    pub fn set_nonblocking(&mut self, flag: bool) -> anyhow::Result<()> {
        if let Some(ws) = &self.inner {
            debug!("set websocket nonblocking {}", flag);
            ws.set_nonblocking(flag)?;
        }
        Ok(())
    }

    pub fn read(&mut self) -> Option<chat::Message> {
        if let Some(ws) = self.inner.as_mut() {
            return match ws.recv_message() {
                Ok(message) => match &message {
                    OwnedMessage::Text(text) => {
                        let event = serde_json::from_str::<chat::Message>(&text);
                        match event {
                            Ok(event) => {
                                debug!("{:?}", event);
                                Some(event)
                            }
                            Err(e) => {
                                error!("{}, {:?}", e, message);
                                None
                            }
                        }
                    }
                    OwnedMessage::Close(_) => {
                        self.inner.take();
                        info!("Remote connection closed");
                        Some(chat::Message::Close)
                    }
                    _ => {
                        debug!("{:?}", message);
                        None
                    }
                },
                Err(websocket::WebSocketError::NoDataAvailable) => {
                    self.inner.take();
                    error!("Remote connection was lost");
                    Some(chat::Message::Close)
                }
                _ => None,
            };
        }
        None
    }

    pub fn send<T: Debug + Serialize>(&mut self, data: T) -> anyhow::Result<()> {
        if let Some(ws) = self.inner.as_mut() {
            let message = serde_json::to_string(&data)?;
            let message = OwnedMessage::Text(message);
            ws.send_message(&message)?;
            debug!("Sending {:?}", data);
        }
        Ok(())
    }

    pub fn close(&mut self) -> anyhow::Result<()> {
        if let Some(ws) = self.inner.as_mut() {
            ws.send_message(&OwnedMessage::Close(None))?;
        }
        Ok(())
    }
}
