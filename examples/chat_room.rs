
use core::fmt;
use std::net::{SocketAddr};
use std::sync::Arc;
use futures::{SinkExt, StreamExt, stream::SplitStream};
use tokio::net::TcpListener;
use anyhow::Result;
use dashmap::DashMap;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_util::codec::{Framed, LinesCodec};
use tracing::{info, warn};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{Layer as _, fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt};




#[derive(Default)]
struct State {
    peers: DashMap<SocketAddr, mpsc::Sender<Arc<Message>>>,
}

struct Peer {
    username: String,
    sender: SplitStream<Framed<TcpStream,LinesCodec>>
}

enum Message {
    UserJoined(String),
    UserLeft(String),
    Chat{
        sender: String,
        content: String,
    }
}




#[tokio::main]
async fn main() -> Result<()> {
    let leayer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(leayer).init();
    
    let state = Arc::new(State::default());

    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).await?;
    info!("listening on {}", addr);

    loop {
        let (stream, addr) = listener.accept().await?;
        info!("{} connected", addr);
        let state_clone = state.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_client(state_clone, stream, addr).await {
                warn!("error handling client {}: {:?}", addr, e);
            }
        });
    }
    Ok(())
}


async fn handle_client(state: Arc<State>, stream: TcpStream, addr: SocketAddr) -> Result<()> {
    let mut framed = Framed::new(stream, LinesCodec::new());
    framed.send("send your name: ").await?;
    let name = match framed.next().await {
        Some(Ok(name)) => name,
        Some(Err(e)) => {
            warn!(" error reading name from {}: {:?}", addr, e);
            return Err(e.into());
        },
        None => return Ok(()),
    };

    let mut peer: Peer = state.add(addr, framed, name).await?;
    let message = Arc::new(Message::user_joined(peer.username.clone()));
    state.broadcast(addr, message).await;

    while let Some(line) = peer.sender.next().await {
        let line = match line {
            Ok(line) => line,
            Err(e) => {
                warn!("error reading from {}: {:?}", addr, e);
                break;
            }
        };
        let message = Arc::new(Message::chat(peer.username.clone(), line));
        state.broadcast(addr, message).await;
    }

    let message = Arc::new(Message::user_left(&peer.username));
    info!("{}", message);
    state.broadcast(addr, message).await;
    
    state.peers.remove(&addr);
    Ok(())
}

/*
    给终端发信息，peer使用接收终端信息，然后打包成message，state把message进行广播，state维护一个mpsc::sender，然后各个peer维护一个mpsc::receiver，state把message发给各个peer，peer接收message后发给终端
*/


impl State {
    async fn broadcast(&self, addr: SocketAddr, message: Arc<Message>) {
        for peer in self.peers.iter() {
            if peer.key() == &addr {
                continue;
            }   
            if let Err(e) = peer.value().send(message.clone()).await {
                warn!("failed to send message to {}: {:?}", peer.key(), e);
            }
        }
    }
    async fn add(&self, addr: SocketAddr, framed: Framed<TcpStream, LinesCodec>, username: String) -> Result<Peer> {
        let (mut stream_sender, stream_rceiver) = framed.split();
        let (tx ,mut rx) = mpsc::channel(100);
        self.peers.insert(addr, tx);
        tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                if let Err(e) = stream_sender.send(message.to_string()).await {
                    warn!("failed to send message to {}: {:?}", addr, e);
                    break;
                }
            }
        });
        Ok(Peer {
            username,
            sender: stream_rceiver,
        })
    }
}

impl Message {
    fn user_joined(name: impl Into<String>) -> Self {
        Message::UserJoined(format!("{} joined the chat", name.into()))
    }

    fn user_left(name: impl Into<String>) -> Self {
        Message::UserLeft(format!("{} left the chat", name.into()   ))
    }

    fn chat(sender: impl Into<String>, content: impl Into<String>) -> Self {
        Message::Chat { sender: sender.into(), content: content.into() }
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Message::UserJoined(message) => write!(f, "{}", message),
            Message::UserLeft(message) => write!(f, "{}", message),
            Message::Chat { sender, content } => write!(f, "{}: {}", sender, content),
        }
    }
}