use async_trait::async_trait;
use bb8::Pool;
use bb8_tiberius::ConnectionManager;
use common::hospital::Patient;

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
}

#[async_trait]
impl PatientRepository for DatabasePatientRepository {
    async fn store_patient(&mut self, patient: &Patient) -> Result<Patient, PatientError> {
        let q = "
            INSERT INTO rust.Patients (Name, HospitalID)
            VALUES (@P1, @P2);
        ";

        let mut conn = self.pool.get()
            .await
            .map_err(PatientError::repository)?;
        
        //                                 todo oops, they don't store HospitalID
        conn.execute(q, &[&patient.name(), &1])
            .await
            .map_err(PatientError::repository)?;
        
        // todo retrieve their ID
        Ok(patient.clone())
    }
}