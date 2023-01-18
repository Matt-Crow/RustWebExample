// model structs are how the program represents the problem domain

// the serde (SERialize DEserialize) crate helps convert data to & from JSON
use serde::{Serialize, Deserialize};

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

    pub fn remove_patient_by_id(&mut self, id: u32) {
        self.patients.retain(|p| p.id.is_some() && p.id.unwrap() != id);
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

    pub fn name(&self) -> String {
        self.name.to_owned()
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