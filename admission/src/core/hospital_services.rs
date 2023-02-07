use super::{hospital_models::{Hospital, Patient}, hospital_repository::{HospitalRepository, RepositoryError}};

pub struct HospitalService {
    repository: Box<dyn HospitalRepository + 'static>
}

impl HospitalService {
    pub fn new(repository: impl HospitalRepository + 'static) -> Self {
        Self {
            repository: Box::new(repository)
        }
    }

    pub async fn get_all_hospitals(&mut self) -> Result<Vec<Hospital>, RepositoryError> {
        self.repository.get_all_hospitals().await
    }

    pub async fn get_hospital_by_name(&mut self, name: &str) -> Result<Option<Hospital>, RepositoryError> {
        self.repository.get_hospital(name).await
    }

    pub async fn admit_patient_to_hospital(&mut self, patient: Patient, hospital_name: &str) -> Result<Hospital, RepositoryError> {
        self.repository.add_patient_to_hospital(hospital_name, patient).await
    }

    pub async fn unadmit_patient_from_hospital(&mut self, patient_id: u32, hospital_name: &str) -> Result<Hospital, RepositoryError> {
        self.repository.remove_patient_from_hospital(patient_id, hospital_name).await
    }
}

#[cfg(test)]
pub mod tests {
    use crate::core::hospital_models::Patient;

    use super::*;
    use async_trait::async_trait;
    use mockall::mock;

    mock! {
        Dummy {

        }

        #[async_trait]
        impl HospitalRepository for Dummy {
            async fn get_all_hospitals(&mut self) -> Result<Vec<Hospital>, RepositoryError>;
            async fn get_hospital(&mut self, name: &str) -> Result<Option<Hospital>, RepositoryError>;
            async fn add_patient_to_hospital(&mut self, hospital_name: &str, patient: Patient) -> Result<Hospital, RepositoryError>;
            async fn remove_patient_from_hospital(&mut self, patient_id: u32, hospital_name: &str) -> Result<Hospital, RepositoryError>;
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

    #[tokio::test]
    async fn get_hospital_by_name_forwards_to_repository() {
        let mut mock = MockDummy::new();
        mock
            .expect_get_hospital()
            .once()
            .returning(|_by| Ok(None));
        let mut sut = HospitalService::new(mock);

        let result = sut.get_hospital_by_name("Foo").await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn add_patient_to_hospital_forwards_to_repository() {
        let mut mock = MockDummy::new();
        mock
            .expect_add_patient_to_hospital()
            .once()
            .returning(|_selector, _patient| Err(RepositoryError::other("")));
        let mut sut = HospitalService::new(mock);

        let result = sut.admit_patient_to_hospital(Patient::new("Foo"), "Bar").await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn unadmit_patient_forwards_to_repository() {
        let mut mock = MockDummy::new();
        mock
            .expect_remove_patient_from_hospital()
            .once()
            .returning(|_, _| Err(RepositoryError::other("")));
        let mut sut = HospitalService::new(mock);

        let result = sut.unadmit_patient_from_hospital(1, "Foo").await;

        assert!(result.is_err());
    }
}