use anyhow::Result;
use std::{
    thread::{self, JoinHandle},
    time,
};
use thiserror::Error;
const N: usize = 5;
#[derive(Debug)]
pub struct Msg {
    pub id: i32,
    pub content: String,
}

#[derive(Error, Debug)]
pub enum MpscError {
    #[error("failed to send message")]
    SendError(#[from] std::sync::mpsc::SendError<Msg>),
}
pub fn main() -> Result<(), MpscError> {
    let (tx, rx) = std::sync::mpsc::channel::<Msg>();

    for i in 0..N {
        let _handler1 = producer(tx.clone(), i as i32);
    }
    drop(tx);
    let handler = thread::spawn(move || {
        while let Ok(rsg) = rx.recv() {
            thread::sleep(time::Duration::from_secs(2));
            println!("Received message: {:?}", rsg);
        }
    });
    handler.join().unwrap();
    Ok(())
}

pub fn producer(my_tx: std::sync::mpsc::Sender<Msg>, id: i32) -> JoinHandle<Result<(), MpscError>> {
    let handler = thread::spawn(move || -> Result<(), MpscError> {
        let msg = Msg {
            id,
            content: format!("Hello from producer {}", id),
        };
        my_tx.send(msg)?;
        Ok(())
    });
    handler
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_mpsc_error() {
            let (tx, rx) = std::sync::mpsc::channel::<Msg>();
            drop(rx); // 关闭接收端，让发送失败

            let handle = producer(tx, 1);
            let result = handle.join().unwrap();

            // 验证错误信息
            if let Err(e) = result {
                assert_eq!(format!("{}", e), "failed to send message");
                println!("成功捕获错误: {}", e);
            }
        }
    }
}
