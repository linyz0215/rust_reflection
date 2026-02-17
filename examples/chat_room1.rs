
use futures::StreamExt;
use core::fmt;
use std::{net::SocketAddr, sync::Arc};

use anyhow::Result;
use dashmap::DashMap;
use futures::{SinkExt, stream::SplitStream};
use tokio::{net::{TcpListener, TcpStream}, sync::mpsc};
use tokio_util::codec::{Framed, LinesCodec};
use tracing::{info, level_filters::LevelFilter, warn};
use tracing_subscriber::{Layer as _, fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt};


#[derive(Default)]
struct State {
    peers: DashMap<SocketAddr, mpsc::Sender<Arc<Message>>>,
}

struct Peer {
    addr: SocketAddr,
    receiver: SplitStream<Framed<TcpStream, LinesCodec>>,
}

enum Message {
    user_joined(String),
    user_left(String),
    Chat {
        name: String,
        content: String,
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();
    let state = Arc::new(State::default());
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).await?;
    info!("Listenening on {}", addr);

    loop {
        let (stream, addr) = listener.accept().await?;
        info!("{} connected", addr);
        let state_cloned = state.clone();
        tokio::spawn( async move {
            if let Err(e) = handler_client(stream, addr, state_cloned).await {
                warn!("Error handling client: {}", e);
            }
        });
    }
    #[allow(unreachable_code)]
    Ok(())
}




async fn handler_client(stream: TcpStream, addr: SocketAddr, state: Arc<State>) -> Result<()> {
    let mut framed = Framed::new(stream, LinesCodec::new());
    framed.send("Enter you name:").await?;
    let name = match framed.next().await {
        Some(Ok(line)) => line,
        Some(Err(e)) => {
            warn!("Error reading name: {}",e);
            return Err(e.into());  
        },
        None => return Ok(()),
    };
    let state_clone = state.clone();
    framed.send(format!("Welcome, {}!", name)).await?;
    let message = Message::user_joined(format!("{} has joined the chat", name));
    state.broadcast(addr,Arc::new(message)).await?;
    let mut peer = state.add(addr, framed, state_clone).await?;
    while let Some(line) = peer.receiver.next().await {
        let line = match line {
            Ok(line) => line,
            Err(e) => {
                warn!("Error reading from {}: {}", addr, e);
                break;
            }
        };
        let message = Message::Chat { name: name.clone(), content: line };
        state.broadcast(addr, Arc::new(message)).await?;
    }
    let message = Message::user_left(name);
    state.broadcast(addr, Arc::new(message)).await?;
    state.peers.remove(&addr);
    Ok(())
}

impl State {
    async fn broadcast(&self, addr: SocketAddr, message: Arc<Message>) -> Result<()> {
        for peer in self.peers.iter() {
            if peer.key() == &addr {
                continue;
            }
            peer.value().send(message.clone()).await?;
        }
        Ok(())
    }

    async fn add(&self, addr: SocketAddr, framed: Framed<TcpStream, LinesCodec>, state: Arc<State>) -> Result<Peer> {
        let (mut stream_sender, stream_receiver) = framed.split();
        let (tx, mut rx) = mpsc::channel(100);
        state.peers.insert(addr, tx);
        tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                if let Err(e) = stream_sender.send(message.to_string()).await {
                    warn!("Error sending message to {}: {}",addr, e);
                    break;
                }
            }

        });
        Ok(Peer {
            addr,
            receiver: stream_receiver,
        })
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Message::user_joined(name) => write!(f, "{}", name),
            Message::user_left(name) => write!(f, "{} left the chat", name),
            Message::Chat { name, content } => write!(f, "{}: {}", name, content),
        }
    }
}

