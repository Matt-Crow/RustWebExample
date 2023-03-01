// model structs are how the program represents the problem domain

use async_trait::async_trait;
use std::fmt::{Debug, Display};
// the serde (SERialize DEserialize) crate helps convert data to & from JSON
use serde::{Serialize, Deserialize};

use crate::{hospital_names::HospitalNames, patients::Patient};

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
        self.id.to_owned()
    }

    pub fn add_patient(&mut self, patient: Patient) {
        self.patients.push(patient);
    }

    pub fn name(&self) -> String {
        self.name.to_owned()
    }

    pub fn patients(&self) -> Vec<Patient> {
        self.patients.clone()
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
pub enum Error {
    ExternalServiceError(String)
}

impl Error {
    pub fn external_service_error(msg: impl Debug) -> Self {
        Self::ExternalServiceError(format!("External service error: {:#?}", msg))
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExternalServiceError(msg) => write!(f, "{}", msg)
        }
    }
}

// todo should I include the full service for both admission & census?
#[async_trait]
pub trait HospitalDataProvider: Send + Sync {
    async fn get_all_hospitals(&self) -> Result<Vec<Hospital>, Error>;
}

#[async_trait]
pub trait HospitalNameProvider: Send + Sync {
    async fn get_all_hospital_names(&self) -> Result<HospitalNames, Error>;
}