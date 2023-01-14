// implements HospitalRepository for an MSSQL database

use std::fs::read_to_string;

use tiberius::{Client, ExecuteResult};
use tokio::net::windows::named_pipe::NamedPipeClient;
use tokio_util::compat::Compat;

use crate::core::{hospital_repository::{HospitalRepository, By, RepositoryError}, hospital_models::{Hospital, Patient}};

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

impl HospitalRepository for DatabaseHospitalRepository {
    fn get_all_hospitals(&self) -> Result<Vec<Hospital>, RepositoryError> {
        todo!()
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