use core::time;
use std::{thread};

use anyhow::Result;
use rand::Rng;
use tokio::{fs, runtime::Builder, time::sleep};
pub fn main() -> Result<()> {
    let handler = thread::spawn(move || {
        //tokio rt
        let rt = Builder::new_current_thread().enable_all().build().unwrap();
        let handler1 = rt.spawn(async {
            println!("future1");
            let st = fs::read_to_string("Cargo.toml").await.unwrap();
            println!("{}", st);
        });

        let handler2 = rt.spawn(async {
            println!("future2");
            sleep(time::Duration::from_secs(3)).await;
            println!("future2 done");
        });
        rt.block_on(async {
            println!("block_on");
            sleep(time::Duration::from_secs(10)).await;
            handler1.await.unwrap();
            handler2.await.unwrap();
            println!("block_on done");
        });
    });

    handler.join().unwrap();
    Ok(())
}