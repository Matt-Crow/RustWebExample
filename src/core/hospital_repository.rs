use std::{fmt::Display, sync::Mutex, collections::HashMap};

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

    /// returns a single hospital according to the given criteria, or None if no
    /// hospital matches, or returns an error when applicable
    fn get_hospital(&self, by: &By) -> Result<Option<Hospital>, RepositoryError>; 
}

pub enum By {
    Id(u32),
    Name(String)
}

pub struct InMemoryHospitalRepository {
    next_id: Mutex<u32>,
    hospitals: Mutex<HashMap<u32, Hospital>>,
    name_to_id: Mutex<HashMap<String, u32>>
}

impl InMemoryHospitalRepository {
    pub fn containing(hospitals: &Vec<Hospital>) -> Self {
        let mut repo = Self {
            next_id: Mutex::new(1),
            hospitals: Mutex::new(HashMap::new()),
            name_to_id: Mutex::new(HashMap::new())
        };

        for hospital in hospitals {
            repo.insert(hospital);
        }

        repo
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
                let id_mutex = self.next_id.lock();
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
}