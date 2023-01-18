use std::{fmt::Display, sync::Mutex, collections::HashMap};
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use super::hospital_models::{Hospital, Patient};

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

#[derive(Debug)]
pub enum NewRepositoryError {
    Other(String),
    Tiberius(tiberius::error::Error)
}

impl NewRepositoryError {
    pub fn other(message: &str) -> Self {
        NewRepositoryError::Other(String::from(message))
    }
}

impl Display for NewRepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NewRepositoryError::Other(message) => write!(f, "Repository Error: {}", message),
            NewRepositoryError::Tiberius(inner) => write!(f, "Tiberius Error: {}", inner)
        }
    }
}

/// designates something as an interface into a backing store of hospitals
#[async_trait] // stable Rust does not yet allow async function in traits, which this fixes
pub trait HospitalRepository: Send + Sync { // must be safe to have multiple threads accessing at the same time

    /// retrieves all hospitals from the backing store, then returns them, or
    /// an error if applicable
    async fn get_all_hospitals(&mut self) -> Result<Vec<Hospital>, NewRepositoryError>;

    /// returns a single hospital with the given name, or returns an error when 
    /// applicable. Note that this returns None if no such hospital exists
    async fn get_hospital(&mut self, name: &str) -> Result<Option<Hospital>, NewRepositoryError>;

    /// adds the given patient to a hospital, if able. Returns an error if the
    /// hospital is not already stored
    async fn add_patient_to_hospital(&mut self, hospital_name: &str, patient: Patient) -> Result<Hospital, NewRepositoryError>;

    /// removes the given patient from the given hospital. Returns an error if
    /// the hospital is not stored. Note this method should be idempotent.
    async fn remove_patient_from_hospital(&mut self, patient_id: u32, hospital_name: &str) -> Result<Hospital, NewRepositoryError>;
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

#[async_trait]
impl HospitalRepository for InMemoryHospitalRepository {
    async fn get_all_hospitals(&mut self) -> Result<Vec<Hospital>, NewRepositoryError> {
        let mutex = &self.hospitals;
        let hospitals = mutex.lock();

        match hospitals {
            Ok(all) => Ok(all.values().cloned().collect()),
            Err(error) => Err(NewRepositoryError::other(&format!("Mutex error: {}", error)))
        }
    }

    async fn get_hospital(&mut self, name: &str) -> Result<Option<Hospital>, NewRepositoryError> {
        let mutex = &self.hospitals;
        let hospitals = mutex.lock().unwrap();

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

    async fn add_patient_to_hospital(&mut self, hospital_name: &str, patient: Patient) -> Result<Hospital, NewRepositoryError>{
        let maybe_hospital = self.get_hospital(hospital_name).await?;
        if maybe_hospital.is_none() {
            return Err(NewRepositoryError::Other(format!("Invalid hospital name: \"{}\"", hospital_name)));
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

    async fn remove_patient_from_hospital(&mut self, patient_id: u32, hospital_name: &str) -> Result<Hospital, NewRepositoryError> {
        let hospital = self.get_hospital(hospital_name).await?;

        match hospital {
            Some(mut h) => {
                h.remove_patient_by_id(patient_id);
                self.insert(&h);
                Ok(h)
            },
            None => Err(NewRepositoryError::other(&format!("Invalid hospital name: \"{}\"", hospital_name)))
        }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::core::hospital_models::Patient;

    use super::*;

    impl InMemoryHospitalRepository {
        pub fn containing_single(hospital: Hospital) -> Self {
            Self::containing(&vec![hospital])
        }
    
        pub fn empty() -> Self {
            Self::containing(&Vec::new())
        }
    }

    #[tokio::test]
    async fn get_all_hospitals_given_empty_has_zero_length() {
        let mut sut = InMemoryHospitalRepository::empty();

        let result = sut.get_all_hospitals().await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn get_all_hospitals_containing_hospitals_returns_them() {
        let h1 = Hospital::new("Foo").with_id(1);
        let h2 = Hospital::new("Bar").with_id(2);
        let hospitals = vec![h1.clone(), h2.clone()];
        let mut sut = InMemoryHospitalRepository::containing(&hospitals);

        let result = sut.get_all_hospitals().await;

        assert!(result.is_ok());

        let found = result.unwrap();
        assert!(found.contains(&h1));
        assert!(found.contains(&h2));
    }

    #[tokio::test]
    async fn get_hospital_given_no_matches_returns_none() {
        let mut sut = InMemoryHospitalRepository::empty();

        let result = sut.get_hospital("foo").await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn get_hospital_by_name_returns_hospital_with_that_name() {
        let name = "Foo";
        let expected = Hospital::new(name).with_id(1); // needs ID for equality
        let mut sut = InMemoryHospitalRepository::containing(&vec![expected.clone()]);

        let result = sut.get_hospital(name).await;
        
        assert!(result.is_ok());
        let ok_result = result.unwrap();
        assert!(ok_result.is_some());
        assert_eq!(expected, ok_result.unwrap());
    }

    #[tokio::test]
    async fn get_hospital_by_name_is_case_insensitive() {
        let name = "Foo";
        let expected = Hospital::new(name).with_id(1);
        let mut sut = InMemoryHospitalRepository::containing(&vec![expected.clone()]);

        let result = sut.get_hospital(&name.to_uppercase()).await;

        assert!(result.is_ok());
        let ok_result = result.unwrap();
        assert!(ok_result.is_some());
        assert_eq!(expected, ok_result.unwrap());
    }

    #[tokio::test]
    async fn add_patient_to_hospital_given_invalid_selector_returns_error() {
        let selector = "Bar";
        let patient = Patient::new("Foo");
        let mut sut = InMemoryHospitalRepository::empty();

        let result = sut.add_patient_to_hospital(selector, patient).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn add_patient_to_hospital_given_valid_selector_updates_hospital() {
        let name = "Foo";
        let hospital = Hospital::new(name);
        let patient = Patient::new("Bar").with_id(1);
        let selector = name;
        let mut sut = InMemoryHospitalRepository::containing(&vec![hospital]);

        let result = sut.add_patient_to_hospital(selector, patient.clone()).await;

        assert!(result.is_ok());
        assert!(result.unwrap().has_patient(&patient));
    }

    #[tokio::test]
    async fn add_patient_to_hospital_sets_patient_id() {
        let name = "Foo";
        let hospital = Hospital::new(name);
        let patient = Patient::new("Bar");
        let selector = name;
        let mut sut = InMemoryHospitalRepository::containing(&vec![hospital]);

        let result = sut.add_patient_to_hospital(selector, patient.clone()).await;

        assert!(result.is_ok());
        assert!(result.unwrap().patients().iter().all(|p| p.id().is_some()));
    }

    #[tokio::test]
    async fn remove_patient_from_hospital_given_invalid_hospital_returns_error() {
        let mut sut = InMemoryHospitalRepository::empty();

        let result = sut.remove_patient_from_hospital(1, "bar").await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn remove_patient_from_hospital_when_patient_not_admitted_is_ok() {
        let name = "Foo";
        let hospital = Hospital::new(name);
        let mut sut = InMemoryHospitalRepository::containing_single(hospital);

        let result = sut.remove_patient_from_hospital(1, name).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn remove_patient_from_hospital_when_patient_admitted_removes_them() {
        let patient_id = 1;
        let patient = Patient::new("Bar").with_id(patient_id);

        let hospital_name = "Foo";
        let mut hospital = Hospital::new(hospital_name);
        hospital.add_patient(patient.clone());

        let mut sut = InMemoryHospitalRepository::containing_single(hospital);

        let result = sut.remove_patient_from_hospital(
            patient_id, 
            hospital_name
        ).await;

        assert!(result.is_ok());
        assert!(!result.unwrap().has_patient(&patient));
    }
}