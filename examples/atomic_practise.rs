use std::thread;

use anyhow::Result;
use test_rust::MetricsAtomic;
use std::sync::atomic::Ordering;
const M: usize = 5;
const N: usize = 3;
pub fn main() {
    let my_string = vec![
        "key1".to_string(),
        "key2".to_string(),
        "key3".to_string(),
        "key4".to_string(),
        "key5".to_string(),
    ];
    let my_metrics = MetricsAtomic::new(my_string);

    for i in 0..M {
        let _ = practise_atomic(my_metrics.clone(), format!("key{}", i).as_str());
    }

    for j in 0..N {
        let _ = practise_atomic(my_metrics.clone(), format!("key{}", j).as_str());
    }

    loop {
        thread::sleep(std::time::Duration::from_secs(5));
        // let snapshot = my_metrics
        //     .iter()
        //     .map(|(k, v)| (k.clone(), v.load(std::sync::atomic::Ordering::Relaxed)))
        //     .collect::<Vec<(String, i32)>>();


        let snapshot = my_metrics
            .iter()
            .map(|entry| (entry.0.clone(), entry.1.load(Ordering::Relaxed)))
            .collect::<Vec<(String, i32)>>();
        println!("{:?}", snapshot);
    }
}

pub fn practise_atomic(metrics: MetricsAtomic, str: &str) -> Result<()> {
    let s = str.to_string();
    thread::spawn(move || -> Result<()> {
        loop {
            thread::sleep(std::time::Duration::from_secs(2));
            metrics
                .inc(s.as_str())
                .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        }
    });
    Ok(())
}
