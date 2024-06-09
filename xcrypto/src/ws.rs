use futures_util::{
    stream::{FusedStream, SplitSink, SplitStream},
    SinkExt, StreamExt, TryStreamExt,
};
use log::*;
use std::{
    fmt::{Debug, Display},
    marker::PhantomData,
    net::SocketAddr,
    time::Duration,
};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use websocket::{tungstenite::Message, MaybeTlsStream, WebSocketStream};

#[derive(Debug)]
pub enum Role {
    Client(WebSocketStream<MaybeTlsStream<TcpStream>>),
    Server(TcpListener),
}

#[derive(Debug)]
pub struct WebSocketReceiver<T>
where
    T: StreamExt + Unpin + Debug,
    T::Item: Debug,
{
    peer: SocketAddr,
    inner: T,
}

impl<T> WebSocketReceiver<T>
where
    T: StreamExt + Unpin + Debug,
    T::Item: Debug,
{
    pub fn new(peer: SocketAddr, inner: T) -> Self {
        Self { peer, inner }
    }

    pub fn addr(&self) -> &SocketAddr {
        &self.peer
    }

    pub async fn recv(&mut self) -> Option<T::Item> {
        let msg = self.inner.next().await;
        msg
    }
}

#[derive(Debug)]
pub struct WebSocketSender<T, Item>
where
    T: SinkExt<Item> + Unpin,
    Item: Debug + Display,
    <T as futures_util::Sink<Item>>::Error: std::error::Error + Send + Sync + 'static,
{
    peer: SocketAddr,
    inner: T,
    phantomdata: PhantomData<Item>,
}

impl<T, Item> WebSocketSender<T, Item>
where
    T: SinkExt<Item> + Unpin,
    Item: Debug + Display,
    <T as futures_util::Sink<Item>>::Error: std::error::Error + Send + Sync + 'static,
{
    pub fn new(peer: SocketAddr, inner: T) -> Self {
        Self {
            peer,
            inner,
            phantomdata: PhantomData,
        }
    }

    pub async fn send(&mut self, msg: Item) -> anyhow::Result<()> {
        self.inner.send(msg).await?;
        Ok(())
    }

    pub fn addr(&self) -> &SocketAddr {
        &self.peer
    }

    pub async fn close(&mut self) -> anyhow::Result<()> {
        self.inner.close().await?;
        Ok(())
    }
}

pub struct WebSocket {
    role: Role,
}

impl WebSocket {
    pub async fn client(addr: &str) -> anyhow::Result<Self> {
        let addr = url::Url::parse(addr)?;
        match tokio::time::timeout(Duration::from_secs(3), async {
            websocket::connect_async(&addr).await
        })
        .await
        {
            Ok(ws) => {
                let (ws, _) = ws?;

                info!("Connect to {}", addr);
                return Ok(Self {
                    role: Role::Client(ws),
                });
            }
            Err(_) => {
                return Err(anyhow::anyhow!("Connection timeout({})", addr));
            }
        };
    }

    pub async fn server(addr: &str) -> anyhow::Result<Self> {
        let addr = url::Url::parse(addr)?;
        let listener = TcpListener::bind(format!(
            "{}:{}",
            addr.host_str().expect("Invalid host"),
            addr.port().expect("Invalid port")
        ))
        .await?;
        info!("Bind address {}", addr);

        Ok(Self {
            role: Role::Server(listener),
        })
    }

    pub fn role(&self) -> &Role {
        &self.role
    }

    pub async fn accept(&self) -> anyhow::Result<(SocketAddr, TcpStreamSender, TcpStreamReceiver)> {
        if let Role::Server(svr) = &self.role {
            let (stream, peer) = svr.accept().await?;

            // let peer = peer.to_string();
            info!("Peer address connect: {}", peer);
            let ws = websocket::accept_async(stream).await?;
            let (write, read) = ws.split();

            return Ok((
                peer.clone(),
                WebSocketSender::new(peer.clone(), write),
                WebSocketReceiver::new(peer.clone(), read),
            ));
        }

        Err(anyhow::anyhow!("This role cannot bind address"))
    }

    pub async fn recv(&mut self) -> anyhow::Result<Option<Message>> {
        if let Role::Client(ws) = &mut self.role {
            return Ok(ws.try_next().await?);
        }
        Ok(None)
    }

    pub async fn send(&mut self, msg: Message) -> anyhow::Result<()> {
        if let Role::Client(ws) = &mut self.role {
            ws.send(msg).await?;
        }
        Ok(())
    }

    pub async fn close(
        &mut self,
        msg: Option<websocket::tungstenite::protocol::CloseFrame<'_>>,
    ) -> anyhow::Result<()> {
        if let Role::Client(ws) = &mut self.role {
            ws.close(msg).await?
        }
        Ok(())
    }

    pub fn is_closed(&self) -> bool {
        if let Role::Client(ws) = &self.role {
            return ws.is_terminated();
        }
        return false;
    }
}

pub type MaybeTlsStreamSender =
    WebSocketSender<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>, Message>;
pub type MaybeTlsStreamReceiver =
    WebSocketReceiver<SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>>;

pub type TcpStreamSender = WebSocketSender<SplitSink<WebSocketStream<TcpStream>, Message>, Message>;
pub type TcpStreamReceiver = WebSocketReceiver<SplitStream<WebSocketStream<TcpStream>>>;
pub type Connection = (
    SocketAddr,
    UnboundedSender<Message>,
    UnboundedReceiver<Message>,
);
