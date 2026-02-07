use std::{collections::HashMap,ops::Deref, sync::{Arc, RwLock}, thread};
use anyhow::Result;
use anyhow::anyhow;
use std::fmt::Display;
#[derive(Clone)]
pub struct MetricsRwLock {
    data: Arc<RwLock<HashMap<String, i32>>>,
}

impl MetricsRwLock {
    pub fn new() -> Self {
        MetricsRwLock {
            data: Arc::new(RwLock::new(HashMap::new()))
        }
    }

    pub fn inc(&self, key: impl Into<String>) -> Result<()> {
        let mut data = self.write().map_err(|e| anyhow!(e.to_string()))?;
        let counter = data.entry(key.into()).or_insert(0);
        *counter += 1;
        Ok(())
    }

    pub fn snapshot(&self) -> Result<HashMap<String, i32>> {
        thread::sleep(std::time::Duration::from_millis(1500)); // 模拟读取数据的耗时 
        Ok(self.read().map_err(|e| anyhow!(e.to_string()))?.clone())
    }
}

impl Deref for MetricsRwLock {
    type Target = Arc<RwLock<HashMap<String, i32>>>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl Display for MetricsRwLock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data = self.data.read().map_err(|_| std::fmt::Error)?;
        for (key, value) in data.iter() {
            writeln!(f, "{}: {}", key, value)?;
        }
        Ok(())
    }
}