// this module is responsible for user-related details
// todo maybe move this to auth project

use std::fmt::Display;

use async_trait::async_trait;
use common::user::User;

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

/// Designates something as a backing store for mapping emails to groups
#[async_trait]
pub trait GroupRepository: Send + Sync {

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
        assert_eq!(email, new_user.email());
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
        assert!(result.unwrap().email() == email);
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
        assert!(user.email() == "foo.bar@baz.qux");
        assert!(user.groups().contains(&String::from("foo")));
        assert!(user.groups().contains(&String::from("bar")));
    }
}