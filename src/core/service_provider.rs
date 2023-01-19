// The Rust ecosystem does not appear to support any standard Dependency
// Injection framework, but once one arises, this module can be replaced.

use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::core::{hospital_services::HospitalService, hospital_repository::HospitalRepository};

use super::users::{UserService, UserRepository, User, UserError};

// one weakness of this implementation of DI is how it must know all the 
// services it must provide, unlike the .NET DI subsystem, which uses generics
pub struct ServiceProvider {
    hospital_service: Mutex<HospitalService>, // wrap services in a mutex for safety
    user_service: Mutex<UserService>
}

pub struct DummyUserRepository {

}

#[async_trait]
impl UserRepository for DummyUserRepository {

    async fn get_user_by_name(&self, _name: &str) -> Result<Option<User>, UserError> {
        todo!();
    }

    async fn insert_user(&self, _user: &User) -> Result<User, UserError> {
        todo!();
    }
}

impl ServiceProvider {
    pub fn new(
        hospital_repository: impl HospitalRepository + Send + Sync + 'static
    ) -> Self {
        Self {
            hospital_service: Mutex::new(HospitalService::new(hospital_repository)),
            user_service: Mutex::new(UserService::new(DummyUserRepository {

            }))
        }
    }

    pub fn hospitals(&self) -> &Mutex<HospitalService> {
        &self.hospital_service
    }

    pub fn users(&self) -> &Mutex<UserService> {
        &self.user_service
    }
}

// these two are needed to specify this is safe to share across threads

unsafe impl Send for ServiceProvider {

}

unsafe impl Sync for ServiceProvider {
    
}