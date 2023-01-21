use std::sync::Arc;

use async_trait::async_trait;
use futures_util::{TryStreamExt, StreamExt, future};
use tiberius::Client;
use tokio::{net::TcpStream, sync::Mutex};
use tokio_util::compat::Compat;

use crate::core::users::{UserRepository, User, UserError};

pub struct DatabaseUserRepository {
    client: Arc<Mutex<Client<Compat<TcpStream>>>>
}

impl DatabaseUserRepository {
    pub fn new(client: Arc<Mutex<Client<Compat<TcpStream>>>>) -> Self {
        Self {
            client
        }
    }
}

struct UserToGroupMapping {
    user_id: i32,
    user_name: String,
    group_name: Option<String>
}

#[async_trait]
impl UserRepository for DatabaseUserRepository {
    async fn get_user_by_name(&mut self, name: &str) ->  Result<Option<User>, UserError> {
        let q = "
        SELECT u.UserID AS 'User ID', u.Name AS 'Username', g.GroupName AS 'Group'
          FROM rust.Users u
               LEFT JOIN
               rust.User_Groups g
               ON
               u.UserID = g.UserID
         WHERE u.Name = @P1;
        ";

        let mut client = self.client.lock().await;
        let result = client.query(q, &[&name])
            .await
            .map_err(|e| UserError::Other(e.to_string()))?;
        
        let mappings = result
            .into_row_stream()
            .into_stream()
            .filter_map(|result| match result {
                Ok(row) => future::ready(Some(row)),
                Err(_) => future::ready(None)
            })
            .map(|row| UserToGroupMapping {
                user_id: row.get(0).expect("user ID should not be null"),
                user_name: row.get::<&str, usize>(1).map(String::from).expect("user name should not be null"),
                group_name: row.get::<&str, usize>(2).map(String::from)
            })
            .collect::<Vec<UserToGroupMapping>>()
            .await;

        if mappings.is_empty() {
            return Ok(None)
        }

        let mut u = User::new(&mappings[0].user_name)
            .with_id(mappings[0].user_id);
        
        mappings.iter()
            .filter_map(|mapping| match &mapping.group_name {
                Some(name) => Some(name),
                None => None
            })
            .for_each(|group_name| {
                u.add_to_group(&group_name);
            });
        
        Ok(Some(u))
    }

    async fn insert_user(&mut self, user: &User) ->  Result<User, UserError> {
        todo!()
    }
}