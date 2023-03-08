use async_trait::async_trait;
use std::{fmt::{Debug, Display}, collections::HashSet};
use serde::{Serialize, Deserialize};

use crate::patient::Patient;

#[derive(Debug)]
#[derive(Serialize, Deserialize)] // allows this to be converted to & from JSON
#[serde(rename_all = "camelCase")]
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

#[async_trait]
pub trait HospitalDataProvider: Send + Sync {
    async fn get_all_hospitals(&self) -> Result<Vec<Hospital>, Error>;
}

// #####################
// # Hospital Services #
// #####################

pub struct GetHospitalNamesRequest;

impl GetHospitalNamesRequest {
    pub fn new() -> Self {
        Self {

        }
    }
}

impl Default for GetHospitalNamesRequest {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetHospitalNamesResponse {
    hospital_names: HashSet<String>
}

impl GetHospitalNamesResponse {
    pub fn new<T: ToString>(names: &HashSet<T>) -> Self {
        Self {
            hospital_names: names.iter().map(ToString::to_string).collect()
        }
    }

    pub fn hospital_names(&self) -> HashSet<String> {
        self.hospital_names.iter().cloned().collect()
    }
}

#[derive(Debug)]
pub enum HospitalError {
    Other,
    ExternalServiceError(String)
}

impl HospitalError {
    pub fn external_service_error<T: ToString>(message: T) -> Self {
        Self::ExternalServiceError(message.to_string())
    }
}

impl Display for HospitalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Other => write!(f, "some other hospital error"),
            Self::ExternalServiceError(ref message) => write!(f, "external service error: {}", message)
        }
    }
}

#[async_trait]
pub trait GetHospitalNames {
    async fn get_hospital_names(&mut self, request: GetHospitalNamesRequest) -> Result<GetHospitalNamesResponse, HospitalError>;
}