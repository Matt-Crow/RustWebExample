use std::sync::Mutex;

use crate::repositories::{anchor_repository::AnchorRepository, in_memory_anchor_repository::InMemoryAnchorRepository};

use super::anchor_service::AnchorService;

pub struct ServiceProvider {
    anchor_service: Mutex<AnchorService>
}

impl ServiceProvider {
    pub fn new(anchor_repository: impl AnchorRepository + Send + Sync + 'static) -> Self {
        Self { 
            anchor_service: Mutex::new(AnchorService::new(anchor_repository))
        }
    }

    pub fn default() -> Self {
        Self::new(InMemoryAnchorRepository::new())
    }

    pub fn anchors(&self) -> &Mutex<AnchorService> {
        &self.anchor_service
    }
}

// these two are needed to specify this is safe to share across threads

unsafe impl Send for ServiceProvider {

}

unsafe impl Sync for ServiceProvider {
    
}