use std::{fmt::Display, sync::Mutex, collections::HashMap};

use actix_web::body::BoxBody;
use actix_web::{error::ResponseError, HttpResponse};
use actix_web::http::StatusCode;
use serde::{Serialize, Deserialize};

use super::hospital_models::{Hospital, Patient};

// todo migrate toward using this
#[derive(Debug)]
pub enum NewRepositoryError {
    HospitalNotFound(By)
}

impl ResponseError for NewRepositoryError {
    fn status_code(&self) -> StatusCode {
        match *self {
            Self::HospitalNotFound(_) => StatusCode::NOT_FOUND
        }
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        HttpResponse::build(self.status_code())
            .body(self.to_string())
    }
}

impl Display for NewRepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::HospitalNotFound(ref selector) => write!(f, "Hospital not found: {}", selector)
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RepositoryError {
    message: String
}

impl RepositoryError {
    pub fn new(message: &str) -> Self {
        Self {
            message: String::from(message)
        }
    }
}

impl Display for RepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Repository error: {}", self.message))
    }
}

/// designates something as an interface into a backing store of hospitals
pub trait HospitalRepository: Send + Sync { // must be safe to have multiple threads accessing at the same time

    /// retrieves all hospitals from the backing store, then returns them, or
    /// an error if applicable
    fn get_all_hospitals(&self) -> Result<Vec<Hospital>, RepositoryError>;

    /// returns a single hospital according to the given criteria, or None if no
    /// hospital matches, or returns an error when applicable
    fn get_hospital(&self, by: &By) -> Result<Option<Hospital>, RepositoryError>;

    /// adds the given patient to a hospital, if able. Returns an error if the
    /// hospital is not already stored
    fn add_patient_to_hospital(&mut self, by: &By, patient: Patient) -> Result<Hospital, RepositoryError>;

    /// removes the given patient from the given hospital. Returns an error if
    /// the hospital is not stored. Note this method should be idempotent.
    fn remove_patient_from_hospital(&mut self, patient_id: u32, hospital_selector: &By) -> Result<Hospital, RepositoryError>;
}

#[derive(Debug)]
pub enum By {
    Id(u32),
    Name(String)
}

impl Display for By {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            By::Id(id) => write!(f, "Hospital with id {}", id),
            By::Name(name) => write!(f, "Hospital with name {}", name)
        }
    }
}

pub struct InMemoryHospitalRepository {
    next_hospital_id: Mutex<u32>,
    next_patient_id: Mutex<u32>,
    hospitals: Mutex<HashMap<u32, Hospital>>,
    name_to_id: Mutex<HashMap<String, u32>>
}

impl InMemoryHospitalRepository {
    pub fn containing(hospitals: &Vec<Hospital>) -> Self {
        let mut repo = Self {
            next_hospital_id: Mutex::new(1),
            next_patient_id: Mutex::new(1),
            hospitals: Mutex::new(HashMap::new()),
            name_to_id: Mutex::new(HashMap::new())
        };

        for hospital in hospitals {
            repo.insert(hospital);
        }

        repo
    }

    pub fn containing_single(hospital: Hospital) -> Self {
        Self::containing(&vec![hospital])
    }

    pub fn empty() -> Self {
        Self::containing(&Vec::new())
    }

    fn insert(&mut self, hospital: &Hospital) {
        let store_me =  match hospital.id() {
            Some(_) => {
                hospital.to_owned()
            },
            None => {
                let id_mutex = self.next_hospital_id.lock();
                let mut next_id = id_mutex.unwrap();
                let temp = hospital.with_id(*next_id);
                *next_id += 1;
                temp
            }
        };

        let hospitals_mutex = self.hospitals.lock();
        let mut hospitals = hospitals_mutex.unwrap();
        hospitals.insert(store_me.id().unwrap(), store_me.clone());

        let index_mutex = self.name_to_id.lock();
        let mut index = index_mutex.unwrap();
        index.insert(Self::sanitize_name(&store_me.name()), store_me.id().unwrap());
    }

    fn sanitize_name(name: &str) -> String {
        name.to_lowercase()
    }
}

impl HospitalRepository for InMemoryHospitalRepository {
    fn get_all_hospitals(&self) -> Result<Vec<Hospital>, RepositoryError> {
        let mutex = &self.hospitals;
        let hospitals = mutex.lock();

        match hospitals {
            Ok(all) => Ok(all.values().cloned().collect()),
            Err(error) => Err(RepositoryError::new(&format!("Mutex error: {}", error)))
        }
    }

    fn get_hospital(&self, by: &By) -> Result<Option<Hospital>, RepositoryError> {
        let mutex = &self.hospitals;
        let hospitals = mutex.lock().unwrap();

        match by {
            By::Id(id) => {
                Ok(hospitals.get(id).map(|ptr| ptr.to_owned()))
            },
            By::Name(ref name) => {
                let sanitized = Self::sanitize_name(name);
                let index_mutex = self.name_to_id.lock();
                let index = index_mutex.unwrap();

                if index.contains_key(&sanitized) {
                    let id = index.get(&sanitized).unwrap();
                    Ok(Some(hospitals.get(id).unwrap().to_owned()))
                } else {
                    Ok(None)
                }
            }
        }
    }

    fn add_patient_to_hospital(&mut self, by: &By, patient: Patient) -> Result<Hospital, RepositoryError>{
        let maybe_hospital = self.get_hospital(by)?;
        if maybe_hospital.is_none() {
            return Err(RepositoryError::new(&format!("Invalid selector: {}", by)));
        }
        let mut hospital = maybe_hospital.unwrap();

        let add_me = match patient.id() {
            Some(_) => patient,
            None => {
                let mutex = self.next_patient_id.lock();
                let mut next_id = mutex.unwrap();
                let temp = patient.with_id(*next_id);
                *next_id += 1;
                temp
            }
        };
        hospital.add_patient(add_me);
        self.insert(&hospital);

        Ok(hospital)
    }

    fn remove_patient_from_hospital(&mut self, patient_id: u32, hospital_selector: &By) -> Result<Hospital, RepositoryError> {
        let hospital = self.get_hospital(hospital_selector)?;

        match hospital {
            Some(mut h) => {
                h.remove_patient_by_id(patient_id);
                self.insert(&h);
                Ok(h)
            },
            None => Err(RepositoryError::new(&format!("Invalid selector: {}", hospital_selector)))
        }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::core::hospital_models::Patient;

    use super::*;

    #[test]
    fn get_all_hospitals_given_empty_has_zero_length() {
        let sut = InMemoryHospitalRepository::empty();

        let result = sut.get_all_hospitals();

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn get_all_hospitals_containing_hospitals_returns_them() {
        let h1 = Hospital::new("Foo").with_id(1);
        let h2 = Hospital::new("Bar").with_id(2);
        let hospitals = vec![h1.clone(), h2.clone()];
        let sut = InMemoryHospitalRepository::containing(&hospitals);

        let result = sut.get_all_hospitals();

        assert!(result.is_ok());

        let found = result.unwrap();
        assert!(found.contains(&h1));
        assert!(found.contains(&h2));
    }

    #[test]
    fn get_hospital_given_no_matches_returns_none() {
        let sut = InMemoryHospitalRepository::empty();

        let result = sut.get_hospital(&By::Id(1));

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn get_hospital_by_id_returns_hospital_with_that_id() {
        let id = 1;
        let expected = Hospital::new("Foo").with_id(id);
        let sut = InMemoryHospitalRepository::containing(&vec![expected.clone()]);

        let result = sut.get_hospital(&By::Id(id));

        assert!(result.is_ok());
        let ok_result = result.unwrap();
        assert!(ok_result.is_some());
        assert_eq!(expected, ok_result.unwrap());
    }

    #[test]
    fn get_hospital_by_name_returns_hospital_with_that_name() {
        let name = "Foo";
        let expected = Hospital::new(name).with_id(1); // needs ID for equality
        let sut = InMemoryHospitalRepository::containing(&vec![expected.clone()]);

        let result = sut.get_hospital(&By::Name(name.to_owned()));
        
        assert!(result.is_ok());
        let ok_result = result.unwrap();
        assert!(ok_result.is_some());
        assert_eq!(expected, ok_result.unwrap());
    }

    #[test]
    fn get_hospital_by_name_is_case_insensitive() {
        let name = "Foo";
        let expected = Hospital::new(name).with_id(1);
        let sut = InMemoryHospitalRepository::containing(&vec![expected.clone()]);

        let result = sut.get_hospital(&By::Name(name.to_uppercase()));

        assert!(result.is_ok());
        let ok_result = result.unwrap();
        assert!(ok_result.is_some());
        assert_eq!(expected, ok_result.unwrap());
    }

    #[test]
    fn add_patient_to_hospital_given_invalid_selector_returns_error() {
        let selector = By::Id(1);
        let patient = Patient::new("Foo");
        let mut sut = InMemoryHospitalRepository::empty();

        let result = sut.add_patient_to_hospital(&selector, patient);

        assert!(result.is_err());
    }

    #[test]
    fn add_patient_to_hospital_given_valid_selector_updates_hospital() {
        let name = "Foo";
        let hospital = Hospital::new(name);
        let patient = Patient::new("Bar").with_id(1);
        let selector = By::Name(name.to_owned());
        let mut sut = InMemoryHospitalRepository::containing(&vec![hospital]);

        let result = sut.add_patient_to_hospital(&selector, patient.clone());

        assert!(result.is_ok());
        assert!(result.unwrap().has_patient(&patient));
    }

    #[test]
    fn add_patient_to_hospital_sets_patient_id() {
        let name = "Foo";
        let hospital = Hospital::new(name);
        let patient = Patient::new("Bar");
        let selector = By::Name(name.to_owned());
        let mut sut = InMemoryHospitalRepository::containing(&vec![hospital]);

        let result = sut.add_patient_to_hospital(&selector, patient.clone());

        assert!(result.is_ok());
        assert!(result.unwrap().patients().iter().all(|p| p.id().is_some()));
    }

    #[test]
    fn remove_patient_from_hospital_given_invalid_hospital_returns_error() {
        let mut sut = InMemoryHospitalRepository::empty();

        let result = sut.remove_patient_from_hospital(1, &By::Id(1));

        assert!(result.is_err());
    }

    #[test]
    fn remove_patient_from_hospital_when_patient_not_admitted_is_ok() {
        let name = "Foo";
        let hospital = Hospital::new(name);
        let mut sut = InMemoryHospitalRepository::containing_single(hospital);

        let result = sut.remove_patient_from_hospital(1, &By::Name(name.to_owned()));

        assert!(result.is_ok());
    }

    #[test]
    fn remove_patient_from_hospital_when_patient_admitted_removes_them() {
        let patient_id = 1;
        let patient = Patient::new("Bar").with_id(patient_id);

        let hospital_name = "Foo";
        let mut hospital = Hospital::new(hospital_name);
        hospital.add_patient(patient.clone());

        let mut sut = InMemoryHospitalRepository::containing_single(hospital);

        let result = sut.remove_patient_from_hospital(
            patient_id, 
            &By::Name(hospital_name.to_owned())
        );

        assert!(result.is_ok());
        assert!(!result.unwrap().has_patient(&patient));
    }
}