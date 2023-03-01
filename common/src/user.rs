// user data

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

/// the system representation of a user
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
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