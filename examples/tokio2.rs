
use std::{thread::{self}};

use anyhow::Result;
use tokio::time;
use tokio::sync::mpsc;
pub struct Msg {
    pub id: i32,
    pub content: String,
}
#[tokio::main]
pub async fn main() -> Result<()> {
    let (tx, rx) = mpsc::channel::<Msg>(32);
    let handler = worker(rx);
    let tx1 = tx.clone();
    tokio::spawn(async move {
        for i in 0..5 {
            let msg = Msg {
                id: i,
                content: format!("Message {}", i),
            };
            println!("id={}, thread={:?}", i, std::thread::current().id());
            tx.send(msg).await?;
            time::sleep(std::time::Duration::from_secs(3)).await;
        }
        Ok::<(), anyhow::Error>(())
    });

    tokio::spawn(async move {
        for i in 5..10 {
            let msg = Msg {
                id: i,
                content: format!("Message {}", i),
            };
            println!("id={}, thread={:?}", i, std::thread::current().id());
            tx1.send(msg).await?;
            time::sleep(std::time::Duration::from_secs(2)).await;
        }
        Ok::<(), anyhow::Error>(())
    });
    handler.join().unwrap();
    Ok(())
}

pub fn worker(mut rx: mpsc::Receiver<Msg>) -> thread::JoinHandle<()> {
    let handler= thread::spawn(move || {
        while let Some(msg) = rx.blocking_recv() {
            println!("Received message: id={}, content={}", msg.id, msg.content);
        }
    });
    handler
}