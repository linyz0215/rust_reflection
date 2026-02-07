use std::thread;
use rand::Rng;
use test_rust::MetricsMutex;
use anyhow::Result;
use anyhow::anyhow;
use std::time;
const M: usize = 5;
const N: usize = 3;
pub fn main() -> Result<()> {
    let my_metrics = MetricsMutex::new();

    for i in 0..M {
        let _ = practise_mutex(my_metrics.clone(), format!("key: {}",i));
    }

    for j in 0..N {
        let _ = practise_mutex2(my_metrics.clone(), format!("key2: {}",j));
    }

    loop {
        thread::sleep(time::Duration::from_secs(5));
        println!("{:?}",my_metrics.snapshot().map_err(|e| anyhow!(e.to_string()))?);
    }
}


pub fn practise_mutex(metrics: MetricsMutex, str: String) -> Result<()> {
    thread::spawn( move || {
        loop {
            let mut rng = rand::rng();
            thread::sleep(time::Duration::from_secs(rng.random_range(1..3)));
            metrics.inc(str.as_str()).map_err(|e| anyhow!(e.to_string()))?;
        }
        #[allow(unreachable_code)]
        Ok::<(), anyhow::Error>(())
    });
    Ok(())
}

pub fn practise_mutex2(metrics: MetricsMutex, str: String) -> Result<()> {
    thread::spawn( move || -> Result<()> {
        loop {
            let mut rng = rand::rng();
            thread::sleep(time::Duration::from_secs(rng.random_range(1..2)));
            metrics.inc(str.as_str()).map_err(|e| anyhow!(e.to_string()))?;
        }    
    });
    Ok(())
}









