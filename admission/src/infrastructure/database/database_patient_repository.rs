use async_trait::async_trait;
use bb8::Pool;
use bb8_tiberius::ConnectionManager;
use common::hospital::{Patient, AdmissionStatus};
use tiberius::ExecuteResult;

use crate::patient_services::{PatientRepository, PatientError};

pub struct DatabasePatientRepository {
    pool: Pool<ConnectionManager> // internally uses an Arc
}

impl DatabasePatientRepository {
    pub fn new(pool: Pool<ConnectionManager>) -> Self {
        Self {
            pool
        }
    }

    pub async fn setup(&mut self) -> Result<ExecuteResult, PatientError> {
        let q = "
            IF OBJECT_ID(N'rust.Waitlist', N'U') IS NOT NULL
                DROP TABLE rust.Waitlist
            
            CREATE TABLE rust.Waitlist (
                PatientID int IDENTITY(1, 1) PRIMARY KEY NOT NULL,
                PatientName varchar(32) NOT NULL
            );
        ";

        let mut conn = self.pool.get()
            .await
            .map_err(PatientError::repository)?;
        
        let result = conn.execute(q, &[])
            .await
            .map_err(PatientError::repository)?;

        Ok(result)
    }

    async fn store_new_patient(&mut self, patient: &Patient) -> Result<Patient, PatientError> {
        let q = "
            INSERT INTO rust.Patients (Name)
            VALUES (@P1);
        ";

        let mut conn = self.pool.get()
            .await
            .map_err(PatientError::repository)?;
        
        conn.execute(q, &[&patient.name()])
            .await
            .map_err(PatientError::repository)?;
        
        // todo set ID
        Ok(patient.clone())
    }

    async fn store_waitlisted_patient(&mut self, patient: &Patient) -> Result<Patient, PatientError> {
        let q = "
            INSERT INTO rust.Patients (Name)
            VALUES (@P1);
        ";

        let mut conn = self.pool.get()
            .await
            .map_err(PatientError::repository)?;
        
        conn.execute(q, &[&patient.name()])
            .await
            .map_err(PatientError::repository)?;
        
        // todo set ID
        Ok(patient.clone())
    }

    async fn store_admitted_patient(&mut self, patient: &Patient, hospital_id: i32) -> Result<Patient, PatientError> {
        let q = "
            INSERT INTO rust.Patients (Name, HospitalID)
            VALUES (@P1, @P2);
        ";

        let mut conn = self.pool.get()
            .await
            .map_err(PatientError::repository)?;
        
        conn.execute(q, &[&patient.name(), &hospital_id])
            .await
            .map_err(PatientError::repository)?;
        
        // todo retrieve their ID
        Ok(patient.clone())
    }
}

#[async_trait]
impl PatientRepository for DatabasePatientRepository {
    async fn store_patient(&mut self, patient: &Patient) -> Result<Patient, PatientError> {
        match patient.status() {
            AdmissionStatus::New => self.store_new_patient(patient).await,
            AdmissionStatus::OnWaitlist => self.store_waitlisted_patient(patient).await,
            AdmissionStatus::Admitted(hospital_id) => self.store_admitted_patient(patient, hospital_id.try_into().unwrap()).await
        }
    }
}