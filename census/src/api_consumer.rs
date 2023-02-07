use async_trait::async_trait;
use common::hospital::{HospitalDataProvider, Hospital, Error};

use crate::http_client::HttpClient;

/// interacts with an external service to get hospital data
pub struct ExternalHospitalDataProvider {
    http_client: HttpClient
}

impl ExternalHospitalDataProvider {
    pub fn new(http_client: HttpClient) -> Self {
        Self {
            http_client
        }
    }
}

#[async_trait]
impl HospitalDataProvider for ExternalHospitalDataProvider {
    async fn get_all_hospitals(&self) ->  Result<Vec<Hospital>, Error> {

        // consume the API provided by the admission project
        let response: Vec<Hospital> = self.http_client.get("/api/v1/hospitals")
            .await
            .map_err(Error::external_service_error)?
            .json()
            .await
            .map_err(Error::external_service_error)?;
        
        Ok(response)
    }
}