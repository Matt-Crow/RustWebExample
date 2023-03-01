use std::{error::Error, fmt::Display};

use async_trait::async_trait;
use common::{patient::{Patient, AdmissionStatus}, complement_service::ComplementService};
use uuid::Uuid;

/// provides services related to patients
pub struct PatientService {
    patient_repository: Box<dyn PatientRepository + 'static>,
    complement_service: ComplementService
}

impl PatientService {
    pub fn new(
        patient_repository: impl PatientRepository + 'static,
        complement_service: ComplementService
    ) -> Self {
        Self {
            patient_repository: Box::new(patient_repository),
            complement_service
        }
    }

    pub async fn get_waitlisted_patients(&mut self) -> Result<Vec<Patient>, PatientError> {
        self.patient_repository.get_waitlisted_patients().await
    }

    /// Adds the given patient to the hospital admission waitlist, if they have
    /// not yet been added to the waitlist and have not yet been admitted to a
    /// hospital. Returns an error if the patient is not added to the waitlist.
    pub async fn add_patient_to_waitlist(&mut self, patient: &Patient) -> Result<Patient, PatientError> {
        match patient.id() {
            Some(id) => Err(PatientError::AlreadyExists(id)),
            None => self.patient_repository.store_patient(&patient.with_status(AdmissionStatus::OnWaitlist).with_random_id()).await
        }
    }

    /// Moves as many patients as possible from the waitlist to a hospital that
    /// can accept them, then returns the updated patients.
    pub async fn admit_patients_from_waitlist(&mut self) -> Result<Vec<Patient>, PatientError> {
        let waitlisted_patients = self.patient_repository.get_waitlisted_patients()
            .await?;

        let mut updated: Vec<Patient> = Vec::new();

        // get the list of hospitals each can be admitted to
        for patient in &waitlisted_patients {
            let t = self.complement_service.compute_complement(patient.disallowed_hospitals()).await;
            
            // pick one from each
            if let Some(hospital) = t.iter().next() {
                updated.push(patient.with_status(AdmissionStatus::admitted(hospital)));
            }
        }

        // update DB
        for patient in &updated {
            self.patient_repository.update_patient_hospital(patient)
                .await?;
        }

        Ok(updated)
    }
}

/// backing store for patients
#[async_trait]
pub trait PatientRepository: Send + Sync {
    async fn store_patient(&mut self, patient: &Patient) -> Result<Patient, PatientError>;
    async fn get_all_patients(&mut self) -> Result<Vec<Patient>, PatientError>;
    async fn get_waitlisted_patients(&mut self) -> Result<Vec<Patient>, PatientError>;
    async fn get_patient_by_id(&mut self, id: Uuid) -> Result<Option<Patient>, PatientError>;
    async fn update_patient_hospital(&mut self, patient: &Patient) -> Result<Patient, PatientError>;
}

#[derive(Debug)]
pub enum PatientError {
    AlreadyExists(Uuid),
    Repository(Box<dyn Error + 'static>),
    Unsupported
}

impl PatientError {
    pub fn repository(inner: impl Error + 'static) -> Self {
        Self::Repository(Box::new(inner))
    }
}

impl Display for PatientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AlreadyExists(id) => write!(f, "Duplicate patient ID: {}", id),
            Self::Repository(inner) => write!(f, "Repository error: {}", inner),
            Self::Unsupported => write!(f, "Unsupported operation")
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use common::complement_service::ComplementProvider;
    use mockall::mock;

    use super::*;

    mock! {
        Patients {

        }

        #[async_trait]
        impl PatientRepository for Patients {
            async fn store_patient(&mut self, patient: &Patient) -> Result<Patient, PatientError>;
            async fn get_all_patients(&mut self) -> Result<Vec<Patient>, PatientError>;
            async fn get_waitlisted_patients(&mut self) -> Result<Vec<Patient>, PatientError>;
            async fn get_patient_by_id(&mut self, id: Uuid) -> Result<Option<Patient>, PatientError>;
            async fn update_patient_hospital(&mut self, patient: &Patient) -> Result<Patient, PatientError>;
        }
    }

    mock! {
        Complements {

        }

        #[async_trait]
        impl ComplementProvider for Complements {
            async fn compute_complement(&self, set: HashSet<String>) -> HashSet<String>;
        }
    }

    #[tokio::test]
    async fn add_patient_to_waitlist_given_an_existing_patient_returns_error() {
        let patient = Patient::new("Foo").with_random_id();
        let repo = MockPatients::new();
        let mut sut = PatientService::new(repo, ComplementService::new(MockComplements::new()));

        let result = sut.add_patient_to_waitlist(&patient).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn add_patient_to_waitlist_given_a_new_patient_stores_in_repository() {
        let patient = Patient::new("Foo");
        let mut repo = MockPatients::new();
        repo.expect_store_patient()
            .returning(|p| Ok(p.with_random_id()));
        let mut sut = PatientService::new(repo, ComplementService::new(MockComplements::new()));

        let result = sut.add_patient_to_waitlist(&patient).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn admit_patients_from_waitlist_given_no_waitlisted_patients_does_not_update() {
        let mut repo = MockPatients::new();
        repo.expect_store_patient()
            .never();
        repo.expect_get_waitlisted_patients()
            .once()
            .return_once(|| Ok(Vec::new()));
        let mut sut = PatientService::new(repo, ComplementService::new(MockComplements::new()));

        let result = sut.admit_patients_from_waitlist().await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn admit_patients_from_waitlist_given_a_waitlisted_patient_adds_them_to_a_hospital() {
        let a_waitlisted_patient = Patient::new("Foo")
            .with_random_id()
            .with_status(AdmissionStatus::OnWaitlist);

        let mut repo = MockPatients::new();
        repo.expect_get_waitlisted_patients()
            .once()
            .return_once(|| Ok(vec![a_waitlisted_patient]));
        repo.expect_update_patient_hospital()
            .once()
            .return_once(|p| Ok(p.to_owned()));

        let mut complements = MockComplements::new();
        complements.expect_compute_complement()
            .once()
            .return_once(|_| {
                let mut h = HashSet::new();
                h.insert(String::from("Foo"));
                h
            });

        let mut sut = PatientService::new(repo, ComplementService::new(complements));

        let result = sut.admit_patients_from_waitlist().await;

        assert!(result.is_ok());
        let updated_patients = result.unwrap();
        assert!(updated_patients.len() == 1);
        assert!(updated_patients.iter().all(|p| p.status().is_admitted()));
    }
}