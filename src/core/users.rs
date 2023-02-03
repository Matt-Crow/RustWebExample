// this module is responsible for user-related details

use std::{collections::HashSet, fmt::Display};

use async_trait::async_trait;
use serde::Serialize;

/// the system representation of a user
#[derive(Debug, Serialize)]
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

    pub fn groups(&self) -> Vec<String> {
        self.groups.iter().cloned().collect()
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
    DuplicateUsername(String),
    Tiberius(tiberius::error::Error)
}

impl UserError {
    pub fn other(message: &str) -> Self {
        Self::Other(String::from(message))
    }

    fn duplicate_username(username: &str) -> Self {
        Self::DuplicateUsername(String::from(username))
    }
}

impl Display for UserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateUsername(_) => write!(f, "todo rm"),
            Self::NotImplemented => write!(f, "not implemented"),
            Self::Other(msg) => write!(f, "Other error: {}", msg),
            Self::Tiberius(inner) => write!(f, "Tiberius error: {}", inner)
        }
    }
}

pub struct UserService {
    user_repository: Box<dyn UserRepository>,
    group_repository: Box<dyn GroupRepository>
}

impl UserService {

    pub fn new<T, U>(user_repository: T, group_repository: U) -> Self 
    where 
        T: UserRepository + 'static,
        U: GroupRepository + 'static
    {
        Self {
            user_repository: Box::new(user_repository),
            group_repository: Box::new(group_repository)
        }
    }

    pub async fn create(&mut self, user: &User) -> Result<User, UserError> {
        match self.get_user_by_name(&user.name()).await? {
            Some(_) => Err(UserError::duplicate_username(&user.name())),
            None => self.user_repository.insert_user(user).await
        }
    }

    pub async fn get_user_by_name(&mut self, name: &str) -> Result<Option<User>, UserError> {
        self.user_repository.get_user_by_name(name).await
    }

    pub async fn get_user_by_email(&mut self, email: &str) -> Result<User, UserError> {
        let mut user = User::new(email);
        let groups = self.group_repository.get_group_by_email(email)
            .await?;
        
        for group in groups {
            user.add_to_group(&group);
        }

        Ok(user)
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

/// Designates something as a backing store for mapping emails to groups
#[async_trait]
pub trait GroupRepository {

    /// Attempts to add a mapping between the given email and group.
    /// It is an error to add a group to a email who already belongs to that 
    /// group.
    async fn add_email_to_group(&mut self, email: &str, group: &str) -> Result<(), UserError>;

    /// Returns all groups associated with the given email.
    /// Note that an email can be associated with no groups.
    async fn get_group_by_email(&mut self, email: &str) -> Result<Vec<String>, UserError>;
}

#[cfg(test)]
pub mod tests {
    use mockall::mock;

    use super::*;

    mock! {
        UserDummy {

        }

        #[async_trait]
        impl UserRepository for UserDummy {
            async fn get_user_by_name(&mut self, name: &str) -> Result<Option<User>, UserError>;
            async fn insert_user(&mut self, user: &User) -> Result<User, UserError>;
        }
    }

    mock! {
        GroupDummy {

        }

        #[async_trait]
        impl GroupRepository for GroupDummy {
            async fn add_email_to_group(&mut self, email: &str, group: &str) -> Result<(), UserError>;
            async fn get_group_by_email(&mut self, email: &str) -> Result<Vec<String>, UserError>;
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
        let mut repo = MockUserDummy::new();
        repo
            .expect_get_user_by_name()
            .once()
            .returning(|name| Err(UserError::duplicate_username(name)));
        repo
            .expect_insert_user()
            .never();
        let mut sut = UserService::new(repo, MockGroupDummy::new());
        let result = sut.create(&user).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn create_user_given_new_username_inserts_in_repository() {
        let user = User::new("Foo");
        let mut repo = MockUserDummy::new();
        repo
            .expect_get_user_by_name()
            .once()
            .returning(|_name| Ok(None));
        repo
            .expect_insert_user()
            .once()
            .returning(|u| Ok(u.with_id(1)));
        let mut sut = UserService::new(repo, MockGroupDummy::new());

        let result = sut.create(&user).await;

        assert!(result.is_ok());
        assert!(result.unwrap().id().is_some());
    }

    #[tokio::test]
    async fn get_user_by_name_forwards_to_repository() {
        let mut repo = MockUserDummy::new();
        repo
            .expect_get_user_by_name()
            .once()
            .returning(|_name| Ok(None));
        let mut sut = UserService::new(repo, MockGroupDummy::new());

        let result = sut.get_user_by_name("Foo").await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn get_user_by_email_given_empty_repositories_is_ok_and_has_email() {
        let mut group_repo = MockGroupDummy::new();
        group_repo
            .expect_get_group_by_email()
            .returning(|_email| Ok(Vec::new()));
        let mut sut = UserService::new(MockUserDummy::new(), group_repo);
        let email = "foo.bar@baz.qux";

        let result = sut.get_user_by_email(email).await;

        assert!(result.is_ok());
        assert!(result.unwrap().name == email);
    }

    #[tokio::test]
    async fn get_user_by_email_given_stored_groups_returns_email_and_groups() {
        let mut group_repo = MockGroupDummy::new();
        group_repo
            .expect_get_group_by_email()
            .returning(|_email| Ok(vec![String::from("foo"), String::from("bar")]));
        let mut sut = UserService::new(MockUserDummy::new(), group_repo);

        let result = sut.get_user_by_email("foo.bar@baz.qux").await;

        assert!(result.is_ok());
        let user = result.unwrap();
        assert!(user.name == "foo.bar@baz.qux");
        assert!(user.groups().contains(&String::from("foo")));
        assert!(user.groups().contains(&String::from("bar")));
    }
}