// model structs are how the program represents the problem domain

use async_trait::async_trait;
use std::{fmt::{Debug, Display}, collections::HashSet};
// the serde (SERialize DEserialize) crate helps convert data to & from JSON
use serde::{Serialize, Deserialize};
use uuid::Uuid;

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

#[derive(Debug, Serialize, Deserialize)]
pub struct Patient {
    id: Option<Uuid>,
    name: String,
    disallow_admission_to: HashSet<String>,
    status: AdmissionStatus
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
            name: name.to_owned(),
            disallow_admission_to: HashSet::new(),
            status: AdmissionStatus::New
        }
    }

    pub fn with_id(&self, id: Uuid) -> Self {
        Self {
            id: Some(id),
            name: self.name.to_owned(),
            disallow_admission_to: self.disallow_admission_to.to_owned(),
            status: self.status.clone()
        }
    }

    pub fn with_random_id(&self) -> Self {
        self.with_id(Uuid::new_v4())
    }

    pub fn with_disallowed_hospitals(
        &self, 
        disallowed_hospitals: &HashSet<String>
    ) -> Self {
        Self {
            id: self.id.to_owned(),
            name: self.name.to_owned(),
            disallow_admission_to: disallowed_hospitals.to_owned(),
            status: self.status.to_owned()
        }
    }

    pub fn with_status(&self, status: AdmissionStatus) -> Self {
        Self {
            id: self.id.to_owned(),
            name: self.name.to_owned(),
            disallow_admission_to: self.disallow_admission_to.to_owned(),
            status
        }
    }

    pub fn add_disallowed_hospital(&mut self, hospital: &str) {
        self.disallow_admission_to.insert(String::from(hospital));
    }

    pub fn name(&self) -> String {
        self.name.to_owned()
    }

    pub fn id(&self) -> Option<Uuid> {
        self.id.to_owned()
    }

    pub fn disallowed_hospitals(&self) -> HashSet<String> {
        self.disallow_admission_to.to_owned()
    }

    pub fn status(&self) -> AdmissionStatus {
        self.status.to_owned()
    }
}

impl Clone for Patient {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            name: self.name.to_string(),
            disallow_admission_to: self.disallow_admission_to.to_owned(),
            status: self.status.clone()
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum AdmissionStatus {
    New,
    OnWaitlist,
    Admitted(u32)
}

impl Clone for AdmissionStatus {
    fn clone(&self) -> Self {
        match self {
            Self::New => Self::New,
            Self::OnWaitlist => Self::OnWaitlist,
            Self::Admitted(id) => Self::Admitted(*id),
        }
    }
}

impl Display for AdmissionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::New => write!(f, "new patient"),
            Self::OnWaitlist => write!(f, "on waitlist"),
            Self::Admitted(id) => write!(f, "admitted to hospital with ID {}", id)
        }
    }
}

pub enum Error {
    ExternalServiceError(String)
}

impl Error {
    pub fn external_service_error(msg: impl Debug) -> Self {
        Self::ExternalServiceError(format!("External service error: {:#?}", msg))
    }
}

impl ToString for Error {
    fn to_string(&self) -> String {
        match self {
            Self::ExternalServiceError(msg) => msg.to_owned()
        }
    }
}

// todo should I include the full service for both admission & census?
#[async_trait]
pub trait HospitalDataProvider: Send + Sync {
    async fn get_all_hospitals(&self) -> Result<Vec<Hospital>, Error>;
}