use std::thread;
use std::time;

use rand::Rng;
use test_rust::MetricsRwLock;
use anyhow::Result;
use anyhow::anyhow;
const M: usize = 5;
const N: usize = 3;
pub fn main() -> Result<()> {
    let my_metrics = MetricsRwLock::new();
    let my_metrics2 = my_metrics.clone();
    for i in 0..M {
        let _ = practise_rwlock(my_metrics.clone(), format!("key1: {}",i).as_str());
    }

    for j in 0..N {
        let _ = practise_rwlock2(my_metrics.clone(), format!("key2: {}",j));
    }



    let handler = thread::spawn( move || -> Result<()>{
        let mut counter = 0;
        loop {
            thread::sleep(time::Duration::from_secs(1));
            counter += 1;
            println!("handler1: {} {:?}", counter, my_metrics.clone().snapshot().map_err(|e| anyhow!(e.to_string()))?);
    }});

    for _ in 0..2 {
        let my_metrics2 = my_metrics2.clone();
        let mut counter2= 0;
        thread::spawn( move || -> Result<()>{
            loop {
                counter2 += 1;
                //let rng = rand::rng();
                thread::sleep(time::Duration::from_millis(4000));
                println!("handler2: {} {:?}", counter2, my_metrics2.snapshot().map_err(|e| anyhow!(e.to_string()))?);       
            }
        });
    }

    let _ =handler.join().unwrap();
    Ok(())
}

pub fn practise_rwlock(metrics: MetricsRwLock, str: &str) -> Result<()> {
    let str = str.to_string();
    thread::spawn( move || -> Result<()> {
        loop {
            let mut rng = rand::rng();
            thread::sleep(time::Duration::from_secs(rng.random_range(1..3)));
            metrics.inc(str.clone()).map_err(|e| anyhow!(e.to_string()))?;
        }
    });
    Ok(())
}

pub fn practise_rwlock2(metrics: MetricsRwLock, str: String) -> Result<()> {
    thread::spawn( move || {
        loop {
            let mut rng = rand::rng();
            thread::sleep(time::Duration::from_secs(rng.random_range(1..2)));
            metrics.inc(str.clone()).map_err(|e| anyhow!(e.to_string()))?;
        }
        #[allow(unreachable_code)]
        Ok::<(), anyhow::Error>(())
    });
    Ok(())
}

