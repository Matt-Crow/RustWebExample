// Implements HospitalRepository for an MSSQL database.
// In most projects, we would use an Object-Relational Mapper (ORM) to convert
// from our program's objects to records in a relational database; however, the
// Rust ecosystem does not currently have an ORM for MSSQL, so we have to
// manually construct the SQL queries ourselves.

use std::{fs::read_to_string, collections::HashMap};

use async_trait::async_trait;
use futures_util::{Future, StreamExt, future, TryStreamExt};
use tiberius::{Client, ExecuteResult};
use tokio::net::windows::named_pipe::NamedPipeClient;
use tokio_util::compat::Compat;

use crate::core::{hospital_repository::{HospitalRepository, By, RepositoryError, NewRepositoryError}, hospital_models::{Hospital, Patient}};

pub struct DatabaseHospitalRepository {
    client: Client<Compat<NamedPipeClient>>
}

impl DatabaseHospitalRepository {
    pub fn new(client: Client<Compat<NamedPipeClient>>) -> Self {
        Self {
            client
        }
    }

    pub async fn setup(&mut self) -> Result<ExecuteResult, RepositoryError> {
        let content = read_to_string("./setup.sql").map_err(|e| RepositoryError::new(&e.to_string()))?;
        println!("Executing setup: \n{}", content);
        let r = self.client.execute(content, &[]).await.map_err(|e| RepositoryError::new(&e.to_string()))?;
        Ok(r)
    }
}

#[derive(Debug)]
struct HospitalPatientMapping {
    hospital_id: i32,
    hospital_name: String,
    patient_id: Option<i32>,
    patient_name: Option<String>
}

#[async_trait]
impl HospitalRepository for DatabaseHospitalRepository {
    async fn get_all_hospitals(&mut self) -> Result<Vec<Hospital>, NewRepositoryError> {
        let q = "
            SELECT h.HospitalID 'Hospital ID', h.Name 'Hospital Name', p.PatientID 'Patient ID', p.Name 'Patient Name'
            FROM rust.Hospitals as h
                LEFT JOIN                   --include hospitals with no patients
                rust.Patients as p
                ON h.HospitalID = p.HospitalID
            ;
        ";

        let query_result = self.client.simple_query(q)
            .await
            .map_err(NewRepositoryError::Tiberius)?;
        
        // map relational to HospitalPatientMapping
        let rows = query_result.into_row_stream()
            .into_stream()
            .map_ok(|row| HospitalPatientMapping {
                hospital_id: row.get(0).expect("hospital ID should be non-null"),
                hospital_name: row.get::<&str, usize>(1).map(String::from).expect("hospital name should be non-null"),
                patient_id: row.get(2),
                patient_name: row.get::<&str, usize>(3).map(String::from)
            })
            .filter(|something| future::ready(something.is_ok()))
            .map(|ok| ok.unwrap())
            .collect::<Vec<HospitalPatientMapping>>()
            .await;

        let mut hm: HashMap<i32, Hospital> = HashMap::new();
        for row in rows {
            if !hm.contains_key(&row.hospital_id) {
                hm.insert(row.hospital_id, Hospital::new(&row.hospital_name).with_id(row.hospital_id.try_into().unwrap()));
            }
            if let (Some(id), Some(name)) = (row.patient_id, row.patient_name) {
                let mut h = hm.get(&row.hospital_id).expect("Hospital with this ID exists by now").to_owned();
                let p = Patient::new(&name).with_id(id.try_into().unwrap());
                h.add_patient(p);
                hm.insert(row.hospital_id, h); // add the updated hospital back in
            }
        }

        Ok(hm.values().map(|href| href.to_owned()).collect())
    }

    fn get_hospital(&self, _by: &By) -> Result<Option<Hospital>, RepositoryError> {
        todo!()
    }

    fn add_patient_to_hospital(&mut self, _by: &By, _patient: Patient) -> Result<Hospital, RepositoryError> {
        todo!()
    }

    fn remove_patient_from_hospital(&mut self, _patient_id: u32, _hospital_selector: &By) -> Result<Hospital, RepositoryError> {
        todo!()
    }
}