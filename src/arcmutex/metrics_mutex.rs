use std::{collections::HashMap, ops::Deref, sync::{Arc, Mutex}};
use anyhow::Result;
use anyhow::anyhow;

#[derive(Clone)]
pub struct  MetricsMutex {
    pub data: Arc<Mutex<HashMap<String, i32>>>,
}

impl MetricsMutex {
    pub fn new() -> Self {
        MetricsMutex {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn inc(&self, key: impl Into<String>) -> Result<()> {
        let mut data = self.lock().map_err(|e| anyhow!(e.to_string()))?;
        let counter = data.entry(key.into()).or_insert(0);
        *counter += 1;
        Ok(())
    }

    pub fn snapshot(&self) -> Result<HashMap<String, i32>> {
        Ok(self.data.lock().map_err(|e| anyhow!(e.to_string()))?.clone())
    } 
}

impl Deref for MetricsMutex {
    type Target = Arc<Mutex<HashMap<String, i32>>>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}