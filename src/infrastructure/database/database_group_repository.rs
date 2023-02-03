use async_trait::async_trait;

use crate::core::users::{GroupRepository, UserError};

pub struct DatabaseGroupRepository {

}

impl DatabaseGroupRepository {
    pub fn new() -> Self {
        Self {
            
        }
    }
}

#[async_trait]
impl GroupRepository for DatabaseGroupRepository {
    async fn add_email_to_group(&mut self, _email: &str, _group: &str) -> Result<(), UserError> {
        Ok(())
    }

    async fn get_group_by_email(&mut self, _email: &str) -> Result<Vec<String>, UserError> {
        Ok(Vec::new())
    }
}