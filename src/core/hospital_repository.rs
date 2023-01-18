use std::fmt::Display;
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
    InvalidHospitalName(String),
    Tiberius(tiberius::error::Error)
}

impl NewRepositoryError {

    pub fn other(message: &str) -> Self {
        Self::Other(String::from(message))
    }
    pub fn invalid_hospital_name(name: &str) -> Self {
        Self::InvalidHospitalName(String::from(name))
    }

    pub fn tiberius(inner: tiberius::error::Error) -> Self {
        Self::Tiberius(inner)
    }
}

impl Display for NewRepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Other(message) => write!(f, "Other error: {}", message),
            Self::InvalidHospitalName(name) => write!(f, "Invalid hospital name: {}", name),
            Self::Tiberius(inner) => write!(f, "Tiberius Error: {}", inner)
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

#[cfg(test)]
pub mod tests {

    #[tokio::test]
    async fn get_all_hospitals_given_empty_has_zero_length() {
        
    }

    #[tokio::test]
    async fn get_all_hospitals_containing_hospitals_returns_them() {
        
    }

    #[tokio::test]
    async fn get_hospital_given_no_matches_returns_none() {
        
    }

    #[tokio::test]
    async fn get_hospital_by_name_returns_hospital_with_that_name() {
        
    }

    #[tokio::test]
    async fn get_hospital_by_name_is_case_insensitive() {
        
    }

    #[tokio::test]
    async fn add_patient_to_hospital_given_invalid_selector_returns_error() {
        
    }

    #[tokio::test]
    async fn add_patient_to_hospital_given_valid_selector_updates_hospital() {
        
    }

    #[tokio::test]
    async fn add_patient_to_hospital_sets_patient_id() {

    }

    #[tokio::test]
    async fn remove_patient_from_hospital_given_invalid_hospital_returns_error() {
        
    }

    #[tokio::test]
    async fn remove_patient_from_hospital_when_patient_not_admitted_is_ok() {
        
    }

    #[tokio::test]
    async fn remove_patient_from_hospital_when_patient_admitted_removes_them() {
        
    }
}