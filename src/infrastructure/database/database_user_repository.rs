use std::sync::Arc;

use async_trait::async_trait;
use bb8::Pool;
use bb8_tiberius::ConnectionManager;
use futures_util::{TryStreamExt, StreamExt, future};
use tiberius::ToSql;

use crate::core::users::{UserRepository, User, UserError};

pub struct DatabaseUserRepository {
    pool: Arc<Pool<ConnectionManager>> // does this need an arc?
}

impl DatabaseUserRepository {
    pub fn new(pool: Pool<ConnectionManager>) -> Self {
        Self {
            pool: Arc::new(pool)
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

        let mut client = self.pool.get()
            .await
            .map_err(|e| UserError::Other(e.to_string()))?;
        
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
                u.add_to_group(group_name);
            });
        
        Ok(Some(u))
    }

    async fn insert_user(&mut self, user: &User) ->  Result<User, UserError> {
        let create_user_q = "
            INSERT INTO rust.Users (Name)
            VALUES
                (@P1)
        ";

        let mut client = self.pool.get()
            .await
            .map_err(|e| UserError::Other(e.to_string()))?;

        let _create_user_result = client.execute(create_user_q, &[&user.name()])
            .await
            .map_err(UserError::Tiberius)?;
        
        let get_user_id_q = "
            SELECT UserId
              FROM rust.Users
             WHERE Name = @P1
        ";
        let created_id: i32 = client.query(get_user_id_q, &[&user.name()])
            .await
            .map_err(UserError::Tiberius)?
            .into_row() // should only have one row, so grab that
            .await
            .map_err(UserError::Tiberius)?
            .expect("should be exactly one ID for user with the given name")
            .get("UserId")
            .expect("should contain UserId column");
        
        let mut insert_params: Vec<&dyn ToSql> = Vec::new();
        insert_params.push(&created_id);
        
        let mut insert_groups_q = String::from("
            INSERT INTO rust.User_Groups (UserId, GroupName)
            VALUES
        ");
        let groups = user.groups();
        for group in groups.iter() {
            insert_params.push(group);
        }
        let records: Vec<String> = user.groups()
            .iter()
            .enumerate() // (i, val)
            .map(|(i, _val)| i + 2)
            .map(|idx| format!("(@P1, @P{})", idx))
            .collect();
        insert_groups_q += &records.join(", ");
        insert_groups_q += ";";

        println!("Query: {}", insert_groups_q);
        let _r = client.execute(insert_groups_q, &insert_params)
            .await
            .map_err(UserError::Tiberius)?;

        Ok(user.with_id(created_id))
    }
}