use async_trait::async_trait;
use bb8::Pool;
use bb8_tiberius::ConnectionManager;

use crate::core::users::{GroupRepository, UserError};

use super::helpers;

pub struct DatabaseGroupRepository {
    pool: Pool<ConnectionManager> // pool internally uses an Arc
}

impl DatabaseGroupRepository {
    pub fn new(pool: Pool<ConnectionManager>) -> Self {
        Self {
            pool
        }
    }

    pub async fn setup(&mut self) -> Result<(), UserError> {
        let q = "
            IF OBJECT_ID(N'rust.EmailGroup', N'U') IS NOT NULL
                DROP TABLE rust.EmailGroup;
            
            CREATE TABLE rust.EmailGroup (
                Email varchar(32) NOT NULL,
                GroupName varchar(32) NOT NULL,
                INDEX IX_EmailGroup_Email (Email),
                INDEX IX_EmailGroup_GroupName (GroupName),
                CONSTRAINT UK_EmailGroupName UNIQUE(Email, GroupName)
            );
        ";
        { // create a scope so conn drops before executing other queries
            let mut conn = self.pool.get()
                .await
                .map_err(UserError::other)?;
            
            conn.execute(q, &[])
                .await
                .map_err(UserError::Tiberius)?;
        }
        self.add_email_to_group("mattcrow19@gmail.com", "admin").await?;
        self.add_email_to_group("mattcrow19@gmail.com", "student assistant").await?;
        self.add_email_to_group("john.doe@gmail.com", "foo").await?;
        
        Ok(())
    }
}

#[async_trait]
impl GroupRepository for DatabaseGroupRepository {
    async fn add_email_to_group(&mut self, email: &str, group: &str) -> Result<(), UserError> {
        let q = "
            INSERT INTO rust.EmailGroup (Email, GroupName)
            VALUES (@P1, @P2);
        ";

        let mut conn = self.pool.get()
            .await
            .map_err(UserError::other)?;
        
        // ensure case insensitive!
        conn.execute(q, &[&email.to_lowercase(), &group.to_lowercase()])
            .await
            .map_err(UserError::Tiberius)?;

        Ok(())
    }

    async fn get_groups_by_email(&mut self, email: &str) -> Result<Vec<String>, UserError> {
        let q = "
            SELECT GroupName
              FROM rust.EmailGroup
             WHERE Email = @P1;
        ";

        let mut conn = self.pool.get()
            .await
            .map_err(UserError::other)?;

        // ensure case insensitive!
        let result = conn.query(q, &[&email.to_lowercase()])
            .await
            .map_err(UserError::Tiberius)?;
        
        let records = helpers::map(
            result,
            |row| row.get::<&str, usize>(0)
                .map(String::from)
                .expect("should contain at least 1 column")
        ).await;
        
        Ok(records)
    }
}