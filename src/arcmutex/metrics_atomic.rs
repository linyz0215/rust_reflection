use std::{collections::HashMap, ops::Deref, sync::{Arc, atomic::{AtomicI32, Ordering}}};
use anyhow::Result;
use anyhow::anyhow;


#[derive(Clone)]
pub struct MetricsAtomic {
    data: Arc<HashMap<String, AtomicI32>>,
}

impl MetricsAtomic {
    pub fn new(key: Vec<String>) -> Self{
        let mut map = HashMap::new();
        for k in key{
            map.insert(k, AtomicI32::new(0));
        }
        MetricsAtomic {
            data: Arc::new(map),
        }
    }

    pub fn inc(&self, key: &str) -> Result<()> {
        let counter = self.get(key).ok_or_else(|| anyhow!("Key not found"))?;
        counter.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

}

impl Deref for MetricsAtomic {
    type Target = Arc<HashMap<String, AtomicI32>>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}