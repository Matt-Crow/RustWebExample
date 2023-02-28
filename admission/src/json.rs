// Converts the hospital model to a more consistent format for transmission as
// JSON

use std::collections::HashSet;

use common::hospital::AdmissionStatus;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct Hospital {
    id: Option<u32>,
    name: String,
    patients: Vec<Patient>
}

impl From<common::hospital::Hospital> for Hospital {
    fn from(model: common::hospital::Hospital) -> Hospital {
        Hospital { 
            id: model.id(), 
            name: model.name(), 
            patients: model.patients()
                .into_iter()
                .map(Patient::from)
                .collect()
        }
    }
}

impl From<Hospital> for common::hospital::Hospital {
    fn from(json: Hospital) -> common::hospital::Hospital {
        let mut h = common::hospital::Hospital::new(&json.name);

        if let Some(id) = json.id {
            h = h.with_id(id);
        }

        for patient in json.patients {
            h.add_patient(patient.into());
        }

        h
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")] // allows ASP.NET to deserialize
pub struct Patient {
    id: Option<Uuid>,
    name: String,

    #[serde(rename = "disallowAdmissionTo")]
    disallow_admission_to: HashSet<String>,

    #[serde(rename = "admittedTo")]
    admitted_to: Option<String>
}

impl From<common::hospital::Patient> for Patient {
    fn from(model: common::hospital::Patient) -> Patient {
        Patient { 
            id: model.id(), 
            name: model.name(), 
            disallow_admission_to: model.disallowed_hospitals(), 
            admitted_to: match model.status() {
                AdmissionStatus::AdmittedTo(ref name) => Some(name.to_owned()),
                _ => None
            }
        }
    }
}

impl From<Patient> for common::hospital::Patient {
    fn from(json: Patient) -> common::hospital::Patient {
        let mut p = common::hospital::Patient::new(&json.name)
            .with_disallowed_hospitals(&json.disallow_admission_to);
        
        if let Some(id) = json.id {
            p = p.with_id(id);
        }

        p.with_status(match json.admitted_to {
            Some(ref name) => AdmissionStatus::AdmittedTo(name.to_owned()),
            None => AdmissionStatus::OnWaitlist
        })
    }
}