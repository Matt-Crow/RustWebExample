use std::collections::{HashSet, HashMap};

use async_trait::async_trait;
use bb8::Pool;
use bb8_tiberius::ConnectionManager;
use common::hospital::{Patient, AdmissionStatus};
use futures_util::Future;
use tiberius::ExecuteResult;

use crate::patient_services::{PatientRepository, PatientError};

use super::helpers;

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
            IF OBJECT_ID(N'rust.Patient_disallowed_hospitals', N'U') IS NOT NULL
                DROP TABLE rust.Patient_disallowed_hospitals;
            
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

            CREATE TABLE rust.Patient_disallowed_hospitals (
                PatientID uniqueidentifier NOT NULL,
                HospitalID int,

                CONSTRAINT UQ_Patient_disallowed_hospitals_PatientID_HospitalID UNIQUE (PatientID, HospitalID),

                CONSTRAINT FK_Patient_disallowed_hospitals_Patients FOREIGN KEY (PatientID)
                    REFERENCES rust.Patients (PatientID)
                    ON DELETE CASCADE,
                
                CONSTRAINT FK_Patient_disallowed_hospitals_Hospitals FOREIGN KEY (HospitalID)
                    REFERENCES rust.Hospitals (HospitalID)
                    --ON DELETE CASCADE
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

        for disallowed_hospital in store_me.disallowed_hospitals() {
            conn.execute("
                INSERT INTO rust.Patient_disallowed_hospitals (PatientID, HospitalID)
                VALUES (@P1, (
                    SELECT HospitalID
                      FROM rust.Hospitals
                     WHERE Name = @P2
                ));
            ", &[&store_me.id().unwrap(), &disallowed_hospital])
                .await
                .map_err(PatientError::repository)?;
        }
        
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

        for disallowed_hospital in store_me.disallowed_hospitals() {
            conn.execute("
                INSERT INTO rust.Patient_disallowed_hospitals (PatientID, HospitalID)
                VALUES (@P1, (
                    SELECT HospitalID
                      FROM rust.Hospitals
                     WHERE Name = @P2
                ));
            ", &[&store_me.id().unwrap(), &disallowed_hospital])
                .await
                .map_err(PatientError::repository)?;
        }
        
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

struct PatientDisallowedHospitalMapping {
    patient_id: uuid::Uuid,
    patient_name: String,
    hospital_id: Option<i32>,
    disallowed: Option<String>
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

    async fn get_all_patients(&mut self) -> Result<Vec<Patient>, PatientError> {
        let q = "
            SELECT p.PatientID 'Patient ID', p.Name 'Patient Name', p.HospitalID 'Hospital ID', d.Name 'Disallowed Hospital Name'
              FROM rust.Patients AS p
                   LEFT JOIN
                   (
                       rust.Patient_disallowed_hospitals AS pdh
                       JOIN
                       rust.Hospitals AS d
                       ON pdh.HospitalID = d.HospitalID
                   )
                   ON p.PatientID = pdh.PatientID
            ;
        ";

        let mut conn = self.pool.get()
            .await
            .map_err(PatientError::repository)?;

        let result = conn.query(q, &[])
            .await
            .map_err(PatientError::repository)?;
        let rows: Vec<PatientDisallowedHospitalMapping> = helpers::map(
            result,
            |row| PatientDisallowedHospitalMapping {
                patient_id: row.get("Patient ID").expect("Patient ID cannot be null"),
                patient_name: row.get::<&str, &str>("Patient Name").map(String::from).expect("Patient name cannot be null"),
                hospital_id: row.get("Hospital ID"),
                disallowed: row.get::<&str, &str>("Disallowed Hospital Name").map(String::from)
            })
            .await;
        
        let mut hm: HashMap<uuid::Uuid, Patient> = HashMap::new();
        for row in rows {
            let e = hm.entry(row.patient_id)
                .or_insert({
                    let np = Patient::new(&row.patient_name)
                        .with_id(row.patient_id);
                    match row.hospital_id {
                        Some(hospital_id) => np.with_status(AdmissionStatus::Admitted(hospital_id.try_into().unwrap())),
                        None => np.with_status(AdmissionStatus::OnWaitlist)
                    }
                });

            if let Some(hospital_name) = row.disallowed {
                e.add_disallowed_hospital(&hospital_name);
            }
        }

        Ok(hm.values().map(|p| p.to_owned()).collect())
    }

    async fn get_patient_by_id(&mut self, id: uuid::Uuid) -> Result<Option<Patient>, PatientError> {
        let q = "
            SELECT p.Name 'Patient Name', p.HospitalID 'Hospital ID', d.Name 'Disallowed Hospital Name'
              FROM rust.Patients AS p
                   LEFT JOIN
                   (
                       rust.Patient_disallowed_hospitals AS pdh
                       JOIN
                       rust.Hospitals AS d
                       ON pdh.HospitalID = d.HospitalID
                   )
                   ON p.PatientID = pdh.PatientID
             WHERE p.PatientID = @P1;
        ";

        let mut conn = self.pool.get()
            .await
            .map_err(PatientError::repository)?;

        let result = conn.query(q, &[&id])
            .await
            .map_err(PatientError::repository)?;

        let rows: Vec<PatientDisallowedHospitalMapping> = helpers::map(
            result,
            |row| PatientDisallowedHospitalMapping {
                patient_id: id,
                patient_name: row.get::<&str, &str>("Patient Name").map(String::from).expect("Patient name cannot be null"),
                hospital_id: row.get("Hospital ID"),
                disallowed: row.get::<&str, &str>("Disallowed Hospital Name").map(String::from)
            })
            .await;
        
        if rows.is_empty() {
            Ok(None)
        } else {
            let mut p = Patient::new(&rows[0].patient_name)
                .with_id(id);
            match rows[0].hospital_id {
                Some(hospital_id) => p = p.with_status(AdmissionStatus::Admitted(hospital_id.try_into().unwrap())),
                None => p = p.with_status(AdmissionStatus::OnWaitlist)
            };

            let disallowed_hospitals: HashSet<String> = rows.iter()
                .filter_map(|mapping| mapping.disallowed.as_ref().map(|h| h.to_owned()))
                .collect();
            p = p.with_disallowed_hospitals(&disallowed_hospitals);

            Ok(Some(p))
        }
    }
}