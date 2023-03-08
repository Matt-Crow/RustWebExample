use std::collections::HashSet;

use async_trait::async_trait;
use common::{complement_service::ComplementProvider, hospital::GetHospitalNamesResponse};



pub struct RemoteComplementProvider {
    url: String
}

impl RemoteComplementProvider {
    pub fn new(url: &str) -> Self {
        Self {
            url: String::from(url)
        }
    }
}

#[async_trait]
impl ComplementProvider for RemoteComplementProvider {
    async fn compute_complement(&self, set: HashSet<String>) -> HashSet<String> {
        let body = GetHospitalNamesResponse::new(&set);
        let response: GetHospitalNamesResponse = reqwest::Client::new()
            .get(self.url.to_owned() + "/complement")
            .json(&body)
            .send()
            .await
            .expect("request should be OK") // "I don't want to be a responsible 
                                            // programmer and handle the error,
                                            // just fail here if it doesn't work"
            .json()
            .await
            .expect("should be able to convert to GetHospitalNamesResponse");
        response.hospital_names()
    }
}