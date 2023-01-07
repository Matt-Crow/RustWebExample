use std::{fmt::Display, sync::Mutex};

use serde::{Serialize, Deserialize};

use super::hospital_models::Hospital;


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
pub trait HospitalRepository {

    /// retrieves all hospitals from the backing store, then returns them, or
    /// an error if applicable
    fn get_all_hospitals(&self) -> Result<Vec<Hospital>, RepositoryError>;
}

pub struct InMemoryHospitalRepository {
    hospitals: Mutex<Vec<Hospital>>
}

impl InMemoryHospitalRepository {
    pub fn containing(hospitals: &Vec<Hospital>) -> Self {
        Self {
            hospitals: Mutex::new(hospitals.to_owned())
        }
    }

    pub fn empty() -> Self {
        Self::containing(&Vec::new())
    }
}

impl HospitalRepository for InMemoryHospitalRepository {
    fn get_all_hospitals(&self) -> Result<Vec<Hospital>, RepositoryError> {
        let mutex = &self.hospitals;
        let hospitals = mutex.lock();

        match hospitals {
            Ok(all) => Ok(all.iter().map(|h| h.clone()).collect()),
            Err(error) => Err(RepositoryError::new(&format!("Mutex error: {}", error.to_string())))
        }
    }
}

#[cfg(test)]
pub mod tests {
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
}