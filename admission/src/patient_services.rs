use std::error::Error;

use async_trait::async_trait;
use common::hospital::Patient;
use uuid::Uuid;

/// provides services related to patients
pub struct PatientService {
    patient_repository: Box<dyn PatientRepository + 'static> 
}

impl PatientService {
    pub fn new(patient_repository: impl PatientRepository + 'static) -> Self {
        Self {
            patient_repository: Box::new(patient_repository)
        }
    }

    /// Adds the given patient to the hospital admission waitlist, if they have
    /// not yet been added to the waitlist and have not yet been admitted to a
    /// hospital. Returns an error if the patient is not added to the waitlist.
    pub async fn add_patient_to_waitlist(&mut self, patient: &Patient) -> Result<Patient, PatientError> {
        match patient.id() {
            Some(id) => Err(PatientError::AlreadyExists(id)),
            None => self.patient_repository.store_patient(patient).await
        }
    }
}

/// backing store for patients
#[async_trait]
pub trait PatientRepository: Send + Sync {
    async fn store_patient(&mut self, patient: &Patient) -> Result<Patient, PatientError>;
}

#[derive(Debug)]
pub enum PatientError {
    AlreadyExists(Uuid),
    Repository(Box<dyn Error + 'static>)
}

impl PatientError {
    pub fn repository(inner: impl Error + 'static) -> Self {
        Self::Repository(Box::new(inner))
    }
}

#[cfg(test)]
mod tests {
    use common::hospital::Patient;
    use mockall::mock;

    use super::*;

    mock! {
        Patients {

        }

        #[async_trait]
        impl PatientRepository for Patients {
            async fn store_patient(&mut self, patient: &Patient) -> Result<Patient, PatientError>;
        }
    }

    #[tokio::test]
    async fn add_patient_to_waitlist_given_an_existing_patient_returns_error() {
        let patient = Patient::new("Foo").with_random_id();
        let repo = MockPatients::new();
        let mut sut = PatientService::new(repo);

        let result = sut.add_patient_to_waitlist(&patient).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn add_patient_to_waitlist_given_a_new_patient_stores_in_repository() {
        let patient = Patient::new("Foo");
        let mut repo = MockPatients::new();
        repo.expect_store_patient()
            .returning(|p| Ok(p.with_random_id()));
        let mut sut = PatientService::new(repo);

        let result = sut.add_patient_to_waitlist(&patient).await;

        assert!(result.is_ok());
    }
}