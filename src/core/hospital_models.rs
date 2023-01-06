// model structs are how the program represents the problem domain

use std::collections::HashMap;

// the serde (SERialize DEserialize) crate helps convert data to & from JSON
use serde::{Serialize, Deserialize};

pub struct Department { // aggregate root of the model
    hospitals: HashMap<String, Hospital>
}

impl Department { // define methods on the Department
    pub fn new() -> Self {
        Self {
            hospitals: HashMap::new()
        }
    }

    pub fn add_hospital(&mut self, name: &str) -> Result<Hospital, InvalidHospitalName> {
        if self.hospitals.contains_key(name) {
            Err(InvalidHospitalName {
                name: name.to_string()
            })
        } else {
            let h = Hospital { id: Some(1), name: name.to_string(), patients: Vec::new() };
            self.hospitals.insert(h.name.to_string(), h.clone()); 
            Ok(h)
        }
    }
}

impl Default for Department {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct InvalidHospitalName {
    name: String
}

#[derive(Serialize, Deserialize)] // allows this to be converted to & from JSON
pub struct Hospital {
    id: Option<u32>, // Option means this could potentially have no ID 
    name: String,
    patients: Vec<Patient>
}

impl Clone for Hospital {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            name: self.name.to_string(),
            patients: self.patients.iter().map(|p| p.to_owned()).collect()
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Patient {
    id: Option<u32>,
    name: String
}

impl Clone for Patient {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            name: self.name.to_string()
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn department_add_hospital_given_name_returns_hospital_with_name() {
        let name = "Foo";
        let mut sut = Department::new();

        let result = sut.add_hospital(name);

        assert!(result.is_ok());
        assert_eq!(name, result.unwrap().name);
    }

    #[test]
    fn department_add_hospital_does_not_allow_duplicate_names() {
        let name = "Foo";
        let mut sut = Department::new();

        let first_hospital_with_name = sut.add_hospital(name);
        let second_hospital_with_name = sut.add_hospital(name);

        assert!(first_hospital_with_name.is_ok());
        assert!(second_hospital_with_name.is_err());
    }
}