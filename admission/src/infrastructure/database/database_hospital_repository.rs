// Implements HospitalRepository for an MSSQL database.
// In most projects, we would use an Object-Relational Mapper (ORM) to convert
// from our program's objects to records in a relational database; however, the
// Rust ecosystem does not currently have an ORM for MSSQL, so we have to
// manually construct the SQL queries ourselves.

use std::{fs::read_to_string, collections::{HashMap, hash_map::Entry::Vacant}, sync::Arc};

use async_trait::async_trait;
use bb8::Pool;
use bb8_tiberius::ConnectionManager;
use futures_util::{StreamExt, future, TryStreamExt};
use tiberius::ExecuteResult;

use crate::{core::hospital_services::{HospitalRepository, RepositoryError}, patient_services::PatientRepository};
use common::hospital::{Hospital, Patient, AdmissionStatus};

use super::database_patient_repository::DatabasePatientRepository;

pub struct DatabaseHospitalRepository {
    pool: Arc<Pool<ConnectionManager>>, // does this need an arc?
    patients: DatabasePatientRepository
}

impl DatabaseHospitalRepository {

    pub fn new(
        pool: Pool<ConnectionManager>
    ) -> Self {
        Self {
            pool: Arc::new(pool.clone()),
            patients: DatabasePatientRepository::new(pool)
        }
    }

    pub async fn setup(&mut self) -> Result<ExecuteResult, RepositoryError> {
        let content = read_to_string("./admission/setup.sql")
            .map_err(|e| RepositoryError::other(&e.to_string()))?;

        let mut conn = self.pool.get()
            .await
            .map_err(RepositoryError::other)?;
                
        let r = conn.execute(content, &[])
            .await
            .map_err(RepositoryError::other)?;
        
        Ok(r)
    }
}

#[derive(Debug)]
struct HospitalPatientMapping {
    hospital_id: i32,
    hospital_name: String,
    patient_id: Option<uuid::Uuid>
}

#[async_trait]
impl HospitalRepository for DatabaseHospitalRepository {
    async fn get_all_hospitals(&mut self) -> Result<Vec<Hospital>, RepositoryError> {
        let q = "
            SELECT h.HospitalID 'Hospital ID', h.Name 'Hospital Name', p.PatientID 'Patient ID'
            FROM rust.Hospitals as h
                 LEFT JOIN -- include hospitals with no patients
                 rust.Patients as p
                 ON h.HospitalID = p.HospitalID
            ;
        ";

        let mut conn = self.pool.get()
            .await
            .map_err(RepositoryError::other)?;

        let query_result = conn.simple_query(q)
            .await
            .map_err(RepositoryError::tiberius)?;
        
        // map relational to HospitalPatientMapping
        let rows = query_result
            .into_row_stream()
            .into_stream()
            .filter_map(|ok_or_not| future::ready(match ok_or_not {
                Ok(row) => Some(row),
                Err(_) => None
            })) // remove not-OK items
            .map(|row| HospitalPatientMapping {
                hospital_id: row.get(0).expect("hospital ID should be non-null"),
                hospital_name: row.get::<&str, usize>(1).map(String::from).expect("hospital name should be non-null"),
                patient_id: row.get(2)
            })
            .collect::<Vec<HospitalPatientMapping>>()
            .await;

        let mut hm: HashMap<i32, Hospital> = HashMap::new();
        for row in rows {
            if let Vacant(entry) = hm.entry(row.hospital_id) {
                entry.insert(Hospital::new(&row.hospital_name).with_id(row.hospital_id.try_into().unwrap()));
            }
            if let Some(id) = row.patient_id {
                let mut h = hm.get(&row.hospital_id).expect("Hospital with this ID exists by now").to_owned();
                let p = self.patients.get_patient_by_id(id)
                    .await
                    .map_err(RepositoryError::other)?
                    .expect("patient should exist for this ID")
                    .with_status(AdmissionStatus::Admitted(row.hospital_id.try_into().unwrap()));
                h.add_patient(p);
                hm.insert(row.hospital_id, h); // add the updated hospital back in
            }
        }

        Ok(hm.values().map(|href| href.to_owned()).collect())
    }

    async fn get_hospital(&mut self, name: &str) -> Result<Option<Hospital>, RepositoryError> {
        let q = "
            SELECT h.HospitalID 'Hospital ID', h.Name 'Hospital Name', p.PatientID 'Patient ID'
            FROM rust.Hospitals as h
                 LEFT JOIN -- include hospitals with no patients
                 rust.Patients as p
                 ON h.HospitalID = p.HospitalID
             WHERE UPPER(h.Name) = @P1
            ;
        ";

        let mut client = self.pool.get()
            .await
            .map_err(RepositoryError::other)?;

        let query_result = client.query(q, &[&name.to_uppercase()])
            .await
            .map_err(RepositoryError::tiberius)?;
        
        let rows = query_result
            .into_row_stream()
            .into_stream()
            .filter_map(|ok_or_not| future::ready(match ok_or_not {
                Ok(row) => Some(row),
                Err(_) => None
            })) // remove not-OK items
            .map(|row| HospitalPatientMapping {
                hospital_id: row.get(0).expect("hospital ID should be non-null"),
                hospital_name: row.get::<&str, usize>(1).map(String::from).expect("hospital name should be non-null"),
                patient_id: row.get(2)
            })
            .collect::<Vec<HospitalPatientMapping>>()
            .await;
        
        if rows.is_empty() {
            // hospital does not exist
            return Ok(None);
        }

        let mut h = Hospital::new(&rows[0].hospital_name).with_id(rows[0].hospital_id.try_into().unwrap());
        for row in rows {
            if let Some(id) = row.patient_id {
                let p = self.patients.get_patient_by_id(id)
                    .await
                    .map_err(RepositoryError::other)?
                    .expect("patient should exist for this ID")
                    .with_status(AdmissionStatus::Admitted(row.hospital_id.try_into().unwrap()));
                h.add_patient(p);
            }
        }
        
        Ok(Some(h))
    }

    async fn add_patient_to_hospital(&mut self, hospital_name: &str, patient: Patient) -> Result<Hospital, RepositoryError> {
        let q = "
            INSERT INTO rust.Patients (PatientID, Name, HospitalID)
            VALUES (@P1, @P2, (
                SELECT HospitalID
                  FROM rust.Hospitals
                 WHERE UPPER(Name) = @P3
            ))
            ;
        ";
        
        {
            // perform the insertion in an inner scope so the borrow of self
            // gets dropped before the call to self.get_hospital
            let mut conn = self.pool.get()
                .await
                .map_err(RepositoryError::other)?;

            let _result = conn.execute(q, &[&patient.id().unwrap(), &patient.name(), &hospital_name.to_uppercase()])
                .await
                .map_err(RepositoryError::tiberius)?;
        }
        
        self.get_hospital(hospital_name)
            .await?
            .ok_or_else(|| RepositoryError::invalid_hospital_name(hospital_name))
    }

    async fn remove_patient_from_hospital(&mut self, patient_id: uuid::Uuid, hospital_name: &str) -> Result<Hospital, RepositoryError> {
        let q = "
            DELETE FROM rust.Patients
             WHERE PatientID = @P1
               AND HospitalID = (
                   SELECT HospitalID
                     FROM rust.Hospitals
                    WHERE UPPER(Name) = @P2
               )
            ;
        ";
        
        {
            let mut conn = self.pool.get()
                .await
                .map_err(RepositoryError::other)?;

            let _result = conn.execute(q, &[&patient_id, &hospital_name.to_uppercase()])
                .await
                .map_err(RepositoryError::tiberius)?;
            // drops borrow of self here
        }

        self.get_hospital(hospital_name)
            .await?
            .ok_or_else(|| RepositoryError::invalid_hospital_name(hospital_name))
    }
}