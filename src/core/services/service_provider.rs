// The Rust ecosystem does not appear to support any standard Dependency
// Injection framework, but once one arises, this module can be replaced.

use std::sync::Mutex;

use crate::core::{repositories::{
    anchor_repository::AnchorRepository, 
    in_memory_anchor_repository::InMemoryAnchorRepository
}, hospital_services::HospitalService, hospital_repository::{HospitalRepository, InMemoryHospitalRepository}, hospital_models::Hospital};

use super::anchor_service::AnchorService;

// one weakness of this implementation of DI is how it must know all the 
// services it must provide, unlike the .NET DI subsystem, which uses generics
pub struct ServiceProvider {
    anchor_service: Mutex<AnchorService>, // wrap services in a mutex for safety
    hospital_service: Mutex<HospitalService>
}

impl ServiceProvider {
    pub fn new(
        anchor_repository: impl AnchorRepository + Send + Sync + 'static,
        hospital_repository: impl HospitalRepository + Send + Sync + 'static
    ) -> Self {
        Self { 
            anchor_service: Mutex::new(AnchorService::new(anchor_repository)),
            hospital_service: Mutex::new(HospitalService::new(hospital_repository))
        }
    }

    pub fn default() -> Self {
        Self::new(
            InMemoryAnchorRepository::new(),
            InMemoryHospitalRepository::containing(&vec![
                Hospital::new("Atascadero"),
                Hospital::new("Coalinga"),
                Hospital::new("Napa"),
                Hospital::new("Metropolitan"),
                Hospital::new("Patton")
            ])
        )
    }

    pub fn anchors(&self) -> &Mutex<AnchorService> {
        &self.anchor_service
    }

    pub fn hospitals(&self) -> &Mutex<HospitalService> {
        &self.hospital_service
    }
}

// these two are needed to specify this is safe to share across threads

unsafe impl Send for ServiceProvider {

}

unsafe impl Sync for ServiceProvider {
    
}