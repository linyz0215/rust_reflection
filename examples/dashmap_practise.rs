use std::{thread, time};

use anyhow::Result;
use anyhow::anyhow;
use rand::Rng;
use test_rust::*;

const M: usize = 5;
const N: usize = 3;
pub fn main() -> Result<()> {
    let my_metrics = MetricsDashMap::new();

    for i in 0..M {
        let _ = practise_dashmap(my_metrics.clone(), format!("key1: {}",i).as_str());
    }

    for j in 0..N {
        let _ = practise_dashmap2(my_metrics.clone(), format!("key2: {}",j));
    }

    loop {
        thread::sleep(time::Duration::from_secs(5));
        let my_iter = my_metrics.iter();
        let snapshot = my_iter.map(|kv| (kv.key().clone(), *kv.value())).collect::<Vec<(String,i32)>>();
        println!("{:?}", snapshot);
    }
    #[allow(unreachable_code)]
    Ok(())
}


pub fn practise_dashmap(metrics: MetricsDashMap, str: &str) -> Result<()> {
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

pub fn practise_dashmap2(metrics: MetricsDashMap, str: String) -> Result<()> {
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