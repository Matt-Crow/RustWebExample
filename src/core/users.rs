// this module is responsible for user-related details

use std::{collections::HashSet, fmt::Display};

use async_trait::async_trait;
use serde::{Serialize, Deserialize};

/// the system representation of a user
#[derive(Debug, Deserialize, Serialize)]
pub struct User {

    /// email must be unique within the user repository
    email: String,

    /// the authorization groups this user belongs to
    groups: HashSet<String>
}

impl User {
    /// creates an user with the given details
    pub fn new(email: &str) -> Self {
        Self {
            email: String::from(email),
            groups: HashSet::new()
        }
    }

    pub fn email(&self) -> String {
        self.email.clone()
    }

    pub fn groups(&self) -> Vec<String> {
        self.groups.iter().cloned().collect()
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
            email: self.email.clone(), 
            groups: self.groups.clone() 
        }
    }
}

#[derive(Debug)]
pub enum UserError {
    Other(String),
    Tiberius(tiberius::error::Error)
}

impl UserError {
    pub fn other(message: impl ToString) -> Self {
        Self::Other(message.to_string())
    }
}

impl Display for UserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Other(msg) => write!(f, "Other error: {}", msg),
            Self::Tiberius(inner) => write!(f, "Tiberius error: {}", inner)
        }
    }
}

pub struct UserService {
    group_repository: Box<dyn GroupRepository>
}

impl UserService {

    pub fn new<T>(group_repository: T) -> Self 
    where
        T: GroupRepository + 'static
    {
        Self {
            group_repository: Box::new(group_repository)
        }
    }

    pub async fn create(&mut self, user: &User) -> Result<User, UserError> {
        // repository does not store user details except groups
        for group in user.groups() {
            self.group_repository.add_email_to_group(&user.email, &group)
                .await?;
        }
        Ok(user.clone())
    }

    pub async fn get_user_by_email(&mut self, email: &str) -> Result<User, UserError> {
        let mut user = User::new(email);
        let groups = self.group_repository.get_groups_by_email(email)
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
    
    /// returns the user with the given email, or None if no such user exists
    async fn get_user_by_email(&mut self, email: &str) -> Result<Option<User>, UserError>;

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
    async fn get_groups_by_email(&mut self, email: &str) -> Result<Vec<String>, UserError>;
}

#[cfg(test)]
pub mod tests {
    use mockall::mock;

    use super::*;


    mock! {
        GroupDummy {

        }

        #[async_trait]
        impl GroupRepository for GroupDummy {
            async fn add_email_to_group(&mut self, email: &str, group: &str) -> Result<(), UserError>;
            async fn get_groups_by_email(&mut self, email: &str) -> Result<Vec<String>, UserError>;
        }
    }

    #[test]
    fn first_parameter_sets_email() {
        let email = "Foo";
        let new_user = User::new(email);
        assert_eq!(email, new_user.email);
    }

    #[tokio::test]
    async fn create_user_given_new_email_group_inserts_in_repository() {
        let mut user = User::new("Foo");
        user.add_to_group("bar");

        let mut repo = MockGroupDummy::new();
        repo.expect_add_email_to_group()
            .once()
            .returning(|_, _| Ok(()));
        let mut sut = UserService::new(repo);

        let result = sut.create(&user).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn get_user_by_email_given_empty_repositories_is_ok_and_has_email() {
        let mut group_repo = MockGroupDummy::new();
        group_repo
            .expect_get_groups_by_email()
            .returning(|_email| Ok(Vec::new()));
        let mut sut = UserService::new(group_repo);
        let email = "foo.bar@baz.qux";

        let result = sut.get_user_by_email(email).await;

        assert!(result.is_ok());
        assert!(result.unwrap().email == email);
    }

    #[tokio::test]
    async fn get_user_by_email_given_stored_groups_returns_email_and_groups() {
        let mut group_repo = MockGroupDummy::new();
        group_repo
            .expect_get_groups_by_email()
            .returning(|_email| Ok(vec![String::from("foo"), String::from("bar")]));
        let mut sut = UserService::new(group_repo);

        let result = sut.get_user_by_email("foo.bar@baz.qux").await;

        assert!(result.is_ok());
        let user = result.unwrap();
        assert!(user.email == "foo.bar@baz.qux");
        assert!(user.groups().contains(&String::from("foo")));
        assert!(user.groups().contains(&String::from("bar")));
    }
}