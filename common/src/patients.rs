use std::{collections::HashSet, fmt::Display};

use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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

/// Designates whether a patients is on a waitlist, admitted to a hospital, or
/// neither.
#[derive(Debug, Deserialize, Serialize)]
pub enum AdmissionStatus {
    New,
    OnWaitlist,
    AdmittedTo(String)
}

impl AdmissionStatus {

    /// admitted to the hospital with the given name
    pub fn admitted(to: &str) -> Self {
        Self::AdmittedTo(String::from(to))
    }

    /// returns whether this patient is admitted to a hospital
    pub fn is_admitted(&self) -> bool {
        matches!(self, Self::AdmittedTo(_))
    }
}

impl Clone for AdmissionStatus {
    fn clone(&self) -> Self {
        match self {
            Self::New => Self::New,
            Self::OnWaitlist => Self::OnWaitlist,
            Self::AdmittedTo(name) => Self::AdmittedTo(name.to_owned())
        }
    }
}

impl Display for AdmissionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::New => write!(f, "new patient"),
            Self::OnWaitlist => write!(f, "on waitlist"),
            Self::AdmittedTo(name) => write!(f, "admitted to {}", name)
        }
    }
}