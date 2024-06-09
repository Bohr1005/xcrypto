use super::handler::Handler;
use crate::market::Market;
use crate::Trade;

use log::*;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::oneshot;
use xcrypto::tungstenite::Message;
use xcrypto::ws::{Connection, TcpStreamReceiver, TcpStreamSender, WebSocket};

pub struct Application {
    listener: WebSocket,
}

impl Application {
    pub async fn new(local: &str) -> anyhow::Result<Self> {
        info!("-------------------- Start --------------------");
        let listener = WebSocket::server(local).await?;
        Ok(Self { listener })
    }

    async fn accept_connect(
        &self,
        tx: &UnboundedSender<Connection>,
        mut stop: oneshot::Receiver<()>,
    ) -> anyhow::Result<()> {
        loop {
            tokio::select! {
                res = self.listener.accept() => {
                    match res {
                        Ok((addr, write, read)) => {
                            let (txreq, rxreq) = unbounded_channel();
                            let (txrsp, rxrsp) = unbounded_channel();

                            tx.send((addr, txrsp, rxreq))?;
                            tokio::spawn(task(txreq, rxrsp, write, read));
                        },
                        Err(e) => error!("{}", e)
                    }

                }
                _ = &mut stop => break,
            }
        }

        Ok(())
    }

    pub async fn keep_running<T: Trade + Send + 'static>(
        self,
        mut market: Market,
        mut trade: T,
    ) -> anyhow::Result<()> {
        let (tx, rx) = unbounded_channel();
        let (stop_tx, stop_rx) = oneshot::channel();

        tokio::spawn(async move {
            let mut handler = Handler::new();

            if let Err(e) = handler.process(rx, &mut market, &mut trade).await {
                error!("{}", e);
            }

            info!("-------------------- Exit --------------------");
            let _ = stop_tx.send(());
        });

        self.accept_connect(&tx, stop_rx).await?;
        Ok(())
    }
}

async fn handle_message(
    read: &mut TcpStreamReceiver,
    tx: &UnboundedSender<Message>,
) -> anyhow::Result<()> {
    if let Some(inner) = read.recv().await {
        let msg = inner?;
        match msg {
            Message::Close(_) => {
                info!("Peer {} Close", read.addr());
                tx.send(msg)?;
                return Err(anyhow::anyhow!("WebSocket Close"));
            }
            _ => tx.send(msg)?,
        }
    }

    Ok(())
}

async fn handle_response(
    rx: &mut UnboundedReceiver<Message>,
    write: &mut TcpStreamSender,
) -> anyhow::Result<()> {
    match rx.recv().await {
        Some(inner) => write.send(inner).await?,
        None => {
            if rx.is_closed() {
                return Err(anyhow::anyhow!("Receiver Close"));
            }
        }
    }
    Ok(())
}

async fn task(
    txreq: UnboundedSender<Message>,
    mut rxrsp: UnboundedReceiver<Message>,
    mut write: TcpStreamSender,
    mut read: TcpStreamReceiver,
) {
    loop {
        tokio::select! {
            res = handle_message(&mut read,&txreq) => {
                if let Err(e) = res {
                    error!("{}", e);
                    break
                }
            },
            res = handle_response(&mut rxrsp, &mut write) => {
                if let Err(e) = res {
                    error!("{}", e);
                    break
                }
            }
        }
    }
    info!("{} Task Finish", write.addr());
}
