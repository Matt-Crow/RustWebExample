use async_trait::async_trait;
use bb8::Pool;
use bb8_tiberius::ConnectionManager;
use common::hospital::{Patient, AdmissionStatus};
use futures_util::Future;
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

    pub async fn setup<F, Fut>(&mut self, setup_hospitals: F) -> Result<ExecuteResult, PatientError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = ()> {
        let q1 = "
            IF OBJECT_ID(N'rust.Patients', N'U') IS NOT NULL
	            DROP TABLE rust.Patients;
        ";

        let mut conn = self.pool.get()
            .await
            .map_err(PatientError::repository)?;
        
        conn.execute(q1, &[])
            .await
            .map_err(PatientError::repository)?;
        
        setup_hospitals().await;

        let q2 = "
            -- if HospitalID is null, patient is on the waitlist
            CREATE TABLE rust.Patients (
                PatientID uniqueidentifier PRIMARY KEY NOT NULL,
                Name varchar(32) NOT NULL,
                HospitalID int,
                CONSTRAINT FK_Patients_Hospitals FOREIGN KEY (HospitalID)
                    REFERENCES rust.Hospitals (HospitalID)
                    ON DELETE CASCADE
            );
            
            INSERT INTO rust.Patients (PatientID, Name, HospitalID)
            VALUES
                (NEWID(), 'John Doe', 1),
                (NEWID(), 'Jane Doe', 1),
                (NEWID(), 'Bob Smith', 2)
            ;
        ";

        let result = conn.execute(q2, &[])
            .await
            .map_err(PatientError::repository)?;

        Ok(result)
    }

    async fn store_new_patient(&mut self, patient: &Patient) -> Result<Patient, PatientError> {
        let store_me = patient
            .with_random_id()
            .with_status(AdmissionStatus::OnWaitlist);

        let q = "
            INSERT INTO rust.Patients (PatientID, Name)
            VALUES (@P1, @P2);
        ";

        let mut conn = self.pool.get()
            .await
            .map_err(PatientError::repository)?;
        
        conn.execute(q, &[&store_me.id().unwrap(), &store_me.name()])
            .await
            .map_err(PatientError::repository)?;
        
        Ok(store_me)
    }

    async fn store_waitlisted_patient(&mut self, patient: &Patient) -> Result<Patient, PatientError> {
        let store_me = patient.clone();

        let q = "
            INSERT INTO rust.Patients (PatientID, Name)
            VALUES (@P1, @P2);
        ";

        let mut conn = self.pool.get()
            .await
            .map_err(PatientError::repository)?;
        
        conn.execute(q, &[&store_me.id().unwrap(), &store_me.name()])
            .await
            .map_err(PatientError::repository)?;
        
        Ok(store_me)
    }

    async fn store_admitted_patient(&mut self, patient: &Patient, hospital_id: i32) -> Result<Patient, PatientError> {
        let q = "
            INSERT INTO rust.Patients (PatientID, Name, HospitalID)
            VALUES (@P1, @P2, @P3);
        ";

        let mut conn = self.pool.get()
            .await
            .map_err(PatientError::repository)?;
        
        conn.execute(q, &[&patient.id().unwrap(), &patient.name(), &hospital_id])
            .await
            .map_err(PatientError::repository)?;
        
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