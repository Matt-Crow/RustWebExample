use std::collections::HashSet;

use serde::{Deserialize, Serialize};

/// "nicer" than responding with just a JSON array
#[derive(Debug, Deserialize, Serialize)]
pub struct HospitalNames {
    names: HashSet<String>
}

impl HospitalNames {
    pub fn new(names: HashSet<String>) -> Self {
        Self {
            names
        }
    }

    pub fn names(&self) -> HashSet<String> {
        self.names.clone()
    }
}