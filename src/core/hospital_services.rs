use super::hospital_models::Hospital;

pub struct HospitalService {

}

impl HospitalService {
    pub fn new() -> Self {
        Self {

        }
    }

    pub fn get_all_hospitals() -> Vec<Hospital> {
        Vec::new()
    }
}

impl Default for HospitalService {
    fn default() -> Self {
        Self::new()
    }
}