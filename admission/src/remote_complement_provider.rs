use std::collections::HashSet;

use async_trait::async_trait;
use common::{complement_service::ComplementProvider, hospital_names::HospitalNames};



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
        let body = HospitalNames::new(set);
        let response: HospitalNames = reqwest::Client::new()
            .get(self.url.to_owned() + "/complement")
            .json(&body)
            .send()
            .await
            .expect("request should be OK")
            .json()
            .await
            .expect("should be able to convert to HospitalNames");
        response.names()
    }
}