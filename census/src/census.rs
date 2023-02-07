use std::{collections::HashMap, fmt::Display};

use common::hospital::HospitalDataProvider;

pub struct CensusService {
    hospital_data_provider: Box<dyn HospitalDataProvider>
}

impl CensusService {
    pub fn new(hospital_data_provider: Box<dyn HospitalDataProvider>) -> Self {
        Self {
            hospital_data_provider
        }
    }

    pub async fn conduct_census(&self) -> Result<CensusResult, String> {
        let hospitals = self.hospital_data_provider.get_all_hospitals()
            .await
            .map_err(|e| e.to_string())?;

        let mut result = CensusResult::new();
        for hospital in hospitals {
            let hospital_name = hospital.name(); // avoid excessive cloning
            for _patient in hospital.patients() {
                result.count(&hospital_name);
            }
        }
        
        Ok(result)
    }
}

#[derive(Debug)]
pub struct CensusResult {
    patients_per_hospital: HashMap<String, usize>,
    total_patients: usize
}

impl CensusResult {
    pub fn new() -> Self {
        Self {
            patients_per_hospital: HashMap::new(),
            total_patients: 0
        }
    }

    /// counts a new patient for the given hospital
    fn count(&mut self, hospital: &str) -> &Self {
        let old = self.patients_per_hospital.get(hospital);
        let new_count = match old {
            Some(old_count) => old_count + 1,
            None => 1
        };
        self.patients_per_hospital.insert(String::from(hospital), new_count);
        self.total_patients += 1;
        self
    }
}

impl Display for CensusResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut msg = String::new();
        msg.push_str("Hospital Patient Census");
        for (hospital, patients) in self.patients_per_hospital.iter() {
            msg.push_str(&format!("\n * {:10}: {}", hospital, patients));
        }
        msg.push_str(&format!("\nTotal: {}", self.total_patients));
        write!(f, "{}", msg)
    }
}