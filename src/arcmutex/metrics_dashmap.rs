use std::{ops::Deref, sync::Arc};

use dashmap::DashMap;
use anyhow::Result;

#[derive(Clone)]
pub struct MetricsDashMap {
    data: Arc<DashMap<String, i32>>,
}


impl MetricsDashMap {
    pub fn new() -> Self {
        MetricsDashMap {
            data: Arc::new(DashMap::new()),
        }
    }

    pub fn inc(&self, key: impl Into<String>) -> Result<()> {
        let mut count = self.entry(key.into()).or_insert(0);
        *count += 1;
        Ok(())
    }
}

impl Deref for MetricsDashMap {
    type Target = Arc<DashMap<String, i32>>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}