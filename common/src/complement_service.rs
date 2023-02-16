use std::collections::HashSet;

use async_trait::async_trait;

/// wraps a ComplementProvider for ease-of-use
pub struct ComplementService(Box<dyn ComplementProvider>);

impl ComplementService {
    pub fn new<T>(inner: T) -> Self
    where
        T: ComplementProvider + 'static 
    {
        Self(Box::new(inner))
    }

    pub async fn compute_complement(&self, set: HashSet<String>) -> HashSet<String> {
        self.0.compute_complement(set).await
    }
}

#[async_trait]
pub trait ComplementProvider: Send + Sync {
    /// returns the complement of the given set according to the universal set
    /// this service is configured for 
    async fn compute_complement(&self, set: HashSet<String>) -> HashSet<String>;
}