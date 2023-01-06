// The Rust ecosystem does not appear to support any standard Dependency
// Injection framework, but once one arises, this module can be replaced.

use std::sync::Mutex;

use crate::core::repositories::{
    anchor_repository::AnchorRepository, 
    in_memory_anchor_repository::InMemoryAnchorRepository
};

use super::anchor_service::AnchorService;

// one weakness of this implementation of DI is how it must know all the 
// services it must provide, unlike the .NET DI subsystem, which uses generics
pub struct ServiceProvider {
    anchor_service: Mutex<AnchorService> // wrap services in a mutex for safety
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