// this module is responsible for user-related details

use std::collections::HashSet;

use async_trait::async_trait;
use serde::Serialize;

/// the system representation of a user
#[derive(Serialize)]
pub struct User {
    /// will be None when the user has not been stored in a repository
    id: Option<i32>,

    /// name must be unique within the user repository
    name: String,

    /// the authorization groups this user belongs to
    groups: HashSet<String>
}

impl User {
    /// creates an user with the given details
    pub fn new(name: &str) -> Self {
        Self {
            id: None,
            name: String::from(name),
            groups: HashSet::new()
        }
    }

    /// returns a copy of this user, except with the given ID
    pub fn with_id(&self, id: i32) -> Self {
        Self {
            id: Some(id),
            name: self.name.to_owned(),
            groups: self.groups.to_owned()
        }
    }

    /// returns a copy of this user's ID
    pub fn id(&self) -> Option<i32> {
        self.id
    }

    /// returns a copy of this user's name
    pub fn name(&self) -> String {
        self.name.to_owned()
    }

    /// checks to see if this user belongs to the given group
    pub fn is_in_group(&self, group: &str) -> bool {
        self.groups.contains(group)
    }

    /// Adds this user to the given group. 
    /// 
    /// It is not an error to add a user to the same group multiple times, but 
    /// every call after the first will have no effect.
    pub fn add_to_group(&mut self, group: &str) {
        self.groups.insert(String::from(group));
    }
}

impl Clone for User {
    fn clone(&self) -> Self {
        Self { 
            id: self.id, 
            name: self.name.clone(), 
            groups: self.groups.clone() 
        }
    }
}

#[derive(Debug)]
pub enum UserError {
    Other(String),
    NotImplemented,
    DuplicateUsername(String)
}

impl UserError {
    pub fn other(message: &str) -> Self {
        Self::Other(String::from(message))
    }

    fn duplicate_username(username: &str) -> Self {
        Self::DuplicateUsername(String::from(username))
    }
}

pub struct UserService {
    repository: Box<dyn UserRepository>
}

impl UserService {

    pub fn new<T>(repository: T) -> Self 
    where T: UserRepository + 'static {
        Self {
            repository: Box::new(repository)
        }
    }

    pub async fn create(&mut self, user: &User) -> Result<User, UserError> {
        match self.get_user_by_name(&user.name()).await? {
            Some(_) => Err(UserError::duplicate_username(&user.name())),
            None => self.repository.insert_user(user).await
        }
    }

    pub async fn get_user_by_name(&mut self, name: &str) -> Result<Option<User>, UserError> {
        self.repository.get_user_by_name(name).await
    }
}

/// Designates something as a backing store for Users.
/// Each method returns a UserError when a problem occurs when interacting with
/// the backing store.
#[async_trait]
//#[automock] // creates test mock of this dependency
pub trait UserRepository {
    
    /// returns the user with the given name, or None if no such user exists
    async fn get_user_by_name(&mut self, name: &str) -> Result<Option<User>, UserError>;

    /// inserts a new user into the backing store
    async fn insert_user(&mut self, user: &User) -> Result<User, UserError>;
}

#[cfg(test)]
pub mod tests {
    use mockall::mock;

    use super::*;

    mock! {
        Dummy {

        }

        #[async_trait]
        impl UserRepository for Dummy {
            async fn get_user_by_name(&mut self, name: &str) -> Result<Option<User>, UserError>;
            async fn insert_user(&mut self, user: &User) -> Result<User, UserError>;
        }
    }

    #[test]
    fn new_user_does_not_have_id() {
        let new_user = User::new("Foo");
        assert!(new_user.id().is_none());
    }

    #[test]
    fn with_id_sets_id() {
        let new_user = User::new("Foo").with_id(1);
        assert_eq!(new_user.id(), Some(1));
    }

    #[test]
    fn first_parameter_sets_name() {
        let name = "Foo";
        let new_user = User::new(name);
        assert_eq!(name, new_user.name());
    }

    #[tokio::test]
    async fn create_user_given_duplicate_username_returns_error() {
        let user = User::new("Foo");
        let mut repo = MockDummy::new();
        repo
            .expect_get_user_by_name()
            .once()
            .returning(|name| Err(UserError::duplicate_username(name)));
        repo
            .expect_insert_user()
            .never();
        let mut sut = UserService::new(repo);
        let result = sut.create(&user).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn create_user_given_new_username_inserts_in_repository() {
        let user = User::new("Foo");
        let mut repo = MockDummy::new();
        repo
            .expect_get_user_by_name()
            .once()
            .returning(|_name| Ok(None));
        repo
            .expect_insert_user()
            .once()
            .returning(|u| Ok(u.with_id(1)));
        let mut sut = UserService::new(repo);

        let result = sut.create(&user).await;

        assert!(result.is_ok());
        assert!(result.unwrap().id().is_some());
    }

    #[tokio::test]
    async fn get_user_by_name_forwards_to_repository() {
        let mut repo = MockDummy::new();
        repo
            .expect_get_user_by_name()
            .once()
            .returning(|_name| Ok(None));
        let mut sut = UserService::new(repo);

        let result = sut.get_user_by_name("Foo").await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }
}