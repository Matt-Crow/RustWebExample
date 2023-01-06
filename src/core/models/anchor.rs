use std::fmt::Display;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Anchor {
    id: Option<u32>,
    name: String,

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
            name: String::from(name),
            years_employed: 0
        }
    }

    /// returns a copy of this news anchor, except with the given name
    pub fn with_name(&self, name: &str) -> Self {
        // note that Option's do not implement Copy, so we must explicitly clone
        // them instead of relying on update syntax, as update syntax would move
        // the Options from self to the return value, thus making self invalid
        Anchor {
            id: self.id,
            name: String::from(name),
            ..*self // copy other properties from this, calling copy() on any
                    // that derive from Copy
        }
    }

    /// returns a copy of this news anchor, except with the given years employed
    pub fn with_years_employed(&self, years_employed: u8) -> Self {
        Anchor {
            id: self.id,
            name: self.name.clone(),
            years_employed
        }
    }

    /// returns a copy of this news anchor, except with the given ID
    pub fn with_id(&self, id: u32) -> Self {
        Anchor { 
            id: Some(id), 
            name: self.name.clone(), 
            ..*self
        }
    }

    pub fn merge(first: &Anchor, second: &Anchor) -> Self {
        let mut id = first.id;
        if let Some(new_id) = second.id {
            id = Some(new_id);
        }

        Anchor {
            id,
            name: second.name.clone(),
            years_employed: second.years_employed
        }
    }

    /// returns an immutable reference to this news anchor's name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// returns how many years this news anchor has been employed
    pub fn years_employed(&self) -> u8 {
        self.years_employed
    }

    /// returns this news anchor's ID
    pub fn id(&self) -> Option<u32> {
        self.id
    }
}

impl Clone for Anchor {
    fn clone(&self) -> Self {
        Self { 
            id: self.id, 
            name: self.name.clone(),
            years_employed: self.years_employed 
        }
    }
}

impl Display for Anchor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}, a news anchor with {} years of experience", 
            self.name, self.years_employed)
    }
}

impl PartialEq for Anchor {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}