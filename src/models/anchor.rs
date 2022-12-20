use std::fmt::Display;

use serde::{Serialize, Deserialize};



#[derive(Serialize, Deserialize)]
pub struct Anchor {
    id: Option<u128>,
    name: String,
    accuracy: Option<f32>,

    #[serde(default = "return_0")]
    years_employed: u8
}

fn return_0() -> u8 {
    0
}

impl Anchor {
    /// creates a new news anchor with the given name
    pub fn new(name: &str) -> Self {
        Anchor {
            id: None,
            name: name.to_string(),
            accuracy: None,
            years_employed: 0
        }
    }

    /// returns a copy of this news anchor, except with the given name
    pub fn with_name(&self) -> Self {
        Anchor {
            id: self.id.clone(),
            name: self.name.to_string(),
            accuracy: self.accuracy.clone(),
            years_employed: self.years_employed
        }
    }

    /// returns a copy of this news anchor, except with the given reporting 
    /// accuracy
    pub fn with_accuracy(&self, accuracy: f32) -> Self {
        Anchor {
            id: self.id.clone(),
            name: self.name.to_string(),
            accuracy: Some(accuracy),
            years_employed: self.years_employed
        }
    }

    /// returns a copy of this news anchor, except with the given years employed
    pub fn with_years_employed(&self, years_employed: u8) -> Self {
        Anchor {
            id: self.id.clone(),
            name: self.name.to_string(),
            accuracy: self.accuracy.clone(),
            years_employed
        }
    }

    /// returns a copy of this news anchor, except with the given ID
    pub fn with_id(&self, id: u128) -> Self {
        Anchor { 
            id: Some(id), 
            name: self.name.to_string(), 
            accuracy: self.accuracy.clone(), 
            years_employed: self.years_employed
        }
    }

    /// returns an immutable reference to this news anchor's name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// returns an immutable reference to this news anchor's reporting accuracy
    pub fn accuracy(&self) -> Option<f32> {
        self.accuracy
    }

    /// returns how many years this news anchor has been employed
    pub fn years_employed(&self) -> u8 {
        self.years_employed
    }

    /// returns this news anchor's ID
    pub fn id(&self) -> Option<u128> {
        self.id
    }
}

impl Display for Anchor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.accuracy {
            Some(accuracy) => {
                write!(f, "{}, a news anchor with {}% accuracy over {} years",
                    self.name, accuracy * 100.0, self.years_employed)
            }
            None => {
                write!(f, "{}, a news anchor who hasn't reported despite being hired for {} years",
                    self.name, self.years_employed)
            }
        }
    }
}