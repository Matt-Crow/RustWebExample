// The Rust ecosystem does not appear to support any standard Dependency
// Injection framework, but once one arises, this module can be replaced.

use std::sync::Mutex;

use crate::core::{hospital_services::HospitalService, hospital_repository::{HospitalRepository, InMemoryHospitalRepository}, hospital_models::Hospital};

// one weakness of this implementation of DI is how it must know all the 
// services it must provide, unlike the .NET DI subsystem, which uses generics
pub struct ServiceProvider {
    hospital_service: Mutex<HospitalService> // wrap services in a mutex for safety
}

impl ServiceProvider {
    pub fn new(
        hospital_repository: impl HospitalRepository + Send + Sync + 'static
    ) -> Self {
        Self {
            hospital_service: Mutex::new(HospitalService::new(hospital_repository))
        }
    }

    pub fn default() -> Self {
        Self::new(
            InMemoryHospitalRepository::containing(&vec![
                Hospital::new("Atascadero"),
                Hospital::new("Coalinga"),
                Hospital::new("Napa"),
                Hospital::new("Metropolitan"),
                Hospital::new("Patton")
            ])
        )
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