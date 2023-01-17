use super::{hospital_models::{Hospital, Patient}, hospital_repository::{RepositoryError, HospitalRepository, By, NewRepositoryError}};

pub struct HospitalService {
    repository: Box<dyn HospitalRepository + 'static>
}

impl HospitalService {
    pub fn new(repository: impl HospitalRepository + 'static) -> Self {
        Self {
            repository: Box::new(repository)
        }
    }

    pub async fn get_all_hospitals(&mut self) -> Result<Vec<Hospital>, NewRepositoryError> {
        self.repository.get_all_hospitals().await
    }

    pub fn get_hospital_by_name(&self, name: &str) -> Result<Option<Hospital>, RepositoryError> {
        self.repository.get_hospital(&By::Name(name.to_owned()))
    }

    pub fn admit_patient_to_hospital(&mut self, patient: Patient, hospital_name: &str) -> Result<Hospital, RepositoryError> {
        self.repository.add_patient_to_hospital(&By::Name(hospital_name.to_owned()), patient)
    }

    pub fn unadmit_patient_from_hospital(&mut self, patient_id: u32, hospital_name: &str) -> Result<Hospital, RepositoryError> {
        self.repository.remove_patient_from_hospital(patient_id, &By::Name(hospital_name.to_owned()))
    }
}

#[cfg(test)]
pub mod tests {
    use crate::core::{hospital_repository::By, hospital_models::Patient};

    use super::*;
    use async_trait::async_trait;
    use mockall::mock;

    mock! {
        Dummy {

        }

        #[async_trait]
        impl HospitalRepository for Dummy {
            async fn get_all_hospitals(&mut self) -> Result<Vec<Hospital>, NewRepositoryError>;
            fn get_hospital(&self, by: &By) -> Result<Option<Hospital>, RepositoryError>;
            fn add_patient_to_hospital(&mut self, by: &By, patient: Patient) -> Result<Hospital, RepositoryError>;
            fn remove_patient_from_hospital(&mut self, patient_id: u32, hospital_selector: &By) -> Result<Hospital, RepositoryError>;
        }
    }

    #[tokio::test]
    async fn get_all_hospitals_forwards_to_repository() {
        let mut mock = MockDummy::new();
        mock
            .expect_get_all_hospitals()
            .once()
            .returning(|| Ok(Vec::new()));
        let mut sut = HospitalService::new(mock);

        let result = sut.get_all_hospitals().await;

        assert!(result.is_ok());
    }

    #[test]
    fn get_hospital_by_name_forwards_to_repository() {
        let mut mock = MockDummy::new();
        mock
            .expect_get_hospital()
            .once()
            .returning(|_by| Ok(None));
        let sut = HospitalService::new(mock);

        let result = sut.get_hospital_by_name("Foo");

        assert!(result.is_ok());
    }

    #[test]
    fn add_patient_to_hospital_forwards_to_repository() {
        let mut mock = MockDummy::new();
        mock
            .expect_add_patient_to_hospital()
            .once()
            .returning(|_selector, _patient| Err(RepositoryError::new("")));
        let mut sut = HospitalService::new(mock);

        let result = sut.admit_patient_to_hospital(Patient::new("Foo"), "Bar");

        assert!(result.is_err());
    }

    #[test]
    fn unadmit_patient_forwards_to_repository() {
        let mut mock = MockDummy::new();
        mock
            .expect_remove_patient_from_hospital()
            .once()
            .returning(|_, _| Err(RepositoryError::new("")));
        let mut sut = HospitalService::new(mock);

        let result = sut.unadmit_patient_from_hospital(1, "Foo");

        assert!(result.is_err());
    }
}