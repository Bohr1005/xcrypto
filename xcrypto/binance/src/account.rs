use crate::ListenKey;
use log::*;
use native_json::DeserializeOwned;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use url::Url;
use xcrypto::{rest::Rest, tungstenite::Message, ws::WebSocket};

async fn fetch_listen_key<T: DeserializeOwned>(rest: &Arc<Rest>, api: &str) -> anyhow::Result<T> {
    let rsp = rest.post(api, &[], false).await?;
    Ok(rsp.json::<T>().await?)
}

async fn ping(rest: Arc<Rest>, api: String) -> anyhow::Result<()> {
    match rest.post(&api, &[], false).await {
        Ok(rsp) => info!("{:?}", rsp),
        Err(e) => error!("{}", e),
    }
    Ok(())
}

pub struct Account<T: ListenKey + DeserializeOwned> {
    addr: String,
    api: String,
    ws: WebSocket,
    time: Instant,
    disconnected: bool,
    rest: Arc<Rest>,
    listenkey: T,
}

impl<T> Account<T>
where
    T: ListenKey + DeserializeOwned,
{
    pub async fn new(addr: &str, api: &str, rest: Arc<Rest>) -> anyhow::Result<Self> {
        let listenkey: T = fetch_listen_key(&rest, api).await?;
        let addr = format!("{}/{}", Url::parse(addr)?.as_str(), listenkey.key());

        info!("Account Websocket: {:?}", addr);
        let ws = WebSocket::client(addr.as_str()).await?;
        Ok(Self {
            addr: addr.into(),
            api: api.into(),
            ws,
            time: Instant::now(),
            disconnected: false,
            rest,
            listenkey,
        })
    }

    pub fn disconnected(&self) -> bool {
        self.disconnected
    }

    pub async fn process(&mut self) -> anyhow::Result<Option<Message>> {
        if self.time.elapsed() >= Duration::from_secs(30 * 60) {
            tokio::spawn(ping(self.rest.clone(), self.api.clone()));
            self.time = Instant::now();
        }

        match self.ws.recv().await? {
            Some(inner) => match &inner {
                Message::Text(_) => return Ok(Some(inner)),
                Message::Ping(ping) => {
                    debug!("{:?}", inner);
                    self.ws.send(Message::Pong(ping.to_owned())).await?;
                }
                _ => (),
            },
            None => {
                if !self.disconnected {
                    error!("account disconnected");
                    self.disconnected = true
                }
            }
        }
        Ok(None)
    }

    pub async fn reconnect(&mut self) -> anyhow::Result<()> {
        if !self.disconnected {
            return Ok(());
        }

        if self.time.elapsed() >= Duration::from_secs(10) {
            self.time = Instant::now();

            self.listenkey = fetch_listen_key(&self.rest, &self.api).await?;
            let addr = format!(
                "{}/{}",
                Url::parse(&self.addr)?.as_str(),
                self.listenkey.key()
            );
            info!("Reconnecting to {}", addr);

            match WebSocket::client(addr.as_str()).await {
                Ok(ws) => {
                    self.ws = ws;
                    self.disconnected = false;
                }
                Err(e) => error!("{}", e),
            }
        }
        Ok(())
    }
}
