// model structs are how the program represents the problem domain

use std::collections::HashMap;

// the serde (SERialize DEserialize) crate helps convert data to & from JSON
use serde::{Serialize, Deserialize};

pub struct Department { // aggregate root of the model
    hospitals: HashMap<String, Hospital>
}

// todo move this logic over to the repository
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
            let next_id = self.hospitals.len() + 1;
            let h = Hospital { 
                id: Some(next_id.try_into().unwrap()), 
                name: name.to_string(), 
                patients: Vec::new() 
            };
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

impl InvalidHospitalName {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string()
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug)]
#[derive(Serialize, Deserialize)] // allows this to be converted to & from JSON
pub struct Hospital {
    id: Option<u32>, // Option means this could potentially have no ID 
    name: String,
    patients: Vec<Patient>
}

impl Hospital {
    pub fn new(name: &str) -> Self {
        Self {
            id: None,
            name: name.to_owned(),
            patients: Vec::new()
        }
    }

    pub fn with_id(&self, id: u32) -> Self {
        Self {
            id: Some(id),
            name: self.name.to_owned(),
            patients: self.patients.clone()
        }
    }

    pub fn id(&self) -> Option<u32> {
        self.id
    }

    pub fn name(&self) -> String {
        self.name.to_owned()
    }

    pub fn patients(&self) -> Vec<Patient> {
        self.patients.to_vec()
    }

    pub fn add_patient(&mut self, patient: Patient) {
        self.patients.push(patient);
    }

    pub fn has_patient(&self, patient: &Patient) -> bool {
        self.patients.contains(patient)
    }
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

impl PartialEq for Hospital {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct Patient {
    id: Option<u32>,
    name: String
}

impl PartialEq for Patient {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Patient {
    pub fn new(name: &str) -> Self {
        Self {
            id: None,
            name: name.to_owned()
        }
    }

    pub fn with_id(&self, id: u32) -> Self {
        Self {
            id: Some(id),
            name: self.name.to_owned()
        }
    }

    pub fn id(&self) -> Option<u32> {
        self.id
    }
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

    #[test]
    fn department_add_hospital_sets_id() {
        let mut sut = Department::new();

        let result = sut.add_hospital("Foo");

        assert!(result.is_ok());
        assert!(result.unwrap().id.is_some());
    }

    #[test]
    fn department_add_hospital_uses_unique_id() {
        let mut sut = Department::new();

        let first_hospital = sut.add_hospital("Foo");
        let second_hospital = sut.add_hospital("Bar");

        assert!(first_hospital.is_ok());
        assert!(second_hospital.is_ok());
        assert_ne!(
            first_hospital.unwrap().id,
            second_hospital.unwrap().id
        );
    }
}