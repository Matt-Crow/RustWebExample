use std::fmt::Display;
use async_trait::async_trait;
use common::hospital::Hospital;

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

    pub async fn unadmit_patient_from_hospital(&mut self, patient_id: uuid::Uuid, hospital_name: &str) -> Result<Hospital, RepositoryError> {
        self.repository.remove_patient_from_hospital(patient_id, hospital_name).await
    }
}

#[derive(Debug)]
pub enum RepositoryError {
    Other(String),
    InvalidHospitalName(String),
    Tiberius(tiberius::error::Error)
}

impl RepositoryError {

    pub fn other(message: impl ToString) -> Self {
        Self::Other(message.to_string())
    }
    pub fn invalid_hospital_name(name: &str) -> Self {
        Self::InvalidHospitalName(String::from(name))
    }

    pub fn tiberius(inner: tiberius::error::Error) -> Self {
        Self::Tiberius(inner)
    }
}

impl Display for RepositoryError {
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
    async fn get_all_hospitals(&mut self) -> Result<Vec<Hospital>, RepositoryError>;

    /// returns a single hospital with the given name, or returns an error when 
    /// applicable. Note that this returns None if no such hospital exists
    async fn get_hospital(&mut self, name: &str) -> Result<Option<Hospital>, RepositoryError>;

    /// removes the given patient from the given hospital. Returns an error if
    /// the hospital is not stored. Note this method should be idempotent.
    async fn remove_patient_from_hospital(&mut self, patient_id: uuid::Uuid, hospital_name: &str) -> Result<Hospital, RepositoryError>;
}

#[cfg(test)]
pub mod tests {
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
            async fn remove_patient_from_hospital(&mut self, patient_id: uuid::Uuid, hospital_name: &str) -> Result<Hospital, RepositoryError>;
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
    async fn unadmit_patient_forwards_to_repository() {
        let mut mock = MockDummy::new();
        mock
            .expect_remove_patient_from_hospital()
            .once()
            .returning(|_, _| Err(RepositoryError::other("")));
        let mut sut = HospitalService::new(mock);

        let result = sut.unadmit_patient_from_hospital(uuid::Uuid::new_v4(), "Foo").await;

        assert!(result.is_err());
    }
}