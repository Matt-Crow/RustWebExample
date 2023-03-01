use std::collections::HashSet;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Patient {
    id: Option<Uuid>,
    name: String,
    disallow_admission_to: HashSet<String>,

    /// the name of the hospital this patient is admitted to, or none if they
    /// are on the waitlist to get into a hospital
    admitted_to: Option<String>
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
            admitted_to: None
        }
    }

    pub fn with_id(&self, id: Uuid) -> Self {
        Self {
            id: Some(id),
            name: self.name.to_owned(),
            disallow_admission_to: self.disallow_admission_to.to_owned(),
            admitted_to: self.admitted_to()
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
            admitted_to: self.admitted_to()
        }
    }

    /// returns a copy of this patient except admitted to the given hospital
    pub fn admit_to(&self, hospital: &str) -> Self {
        Self {
            id: self.id(),
            name: self.name(),
            disallow_admission_to: self.disallowed_hospitals(),
            admitted_to: Some(hospital.to_owned())
        }
    }

    /// returns a copy of this patient, but on the waitlist
    pub fn waitlisted(&self) -> Self {
        Self {
            id: self.id(),
            name: self.name(),
            disallow_admission_to: self.disallowed_hospitals(),
            admitted_to: None
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

    /// returns the name of the hospital this patient is admitted to, or None if
    /// they are not yet admitted to a hospital, and are thus on the waitlist.
    pub fn admitted_to(&self) -> Option<String> {
        self.admitted_to.to_owned()
    }

    /// returns whether this patient is admitted to a hospital
    pub fn is_admitted(&self) -> bool {
        self.admitted_to().is_some()
    }

    /// returns whether this patient is on the waitlist to be admitted to a
    /// hospital
    pub fn is_waitlisted(&self) -> bool {
        self.admitted_to().is_none()
    }
}

impl Clone for Patient {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            name: self.name.to_string(),
            disallow_admission_to: self.disallow_admission_to.to_owned(),
            admitted_to: self.admitted_to.clone()
        }
    }
}