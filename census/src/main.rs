use common::user::User;

use crate::{census::CensusService, api_consumer::ExternalHospitalDataProvider, http_client::HttpClient};

mod api_consumer;
mod http_client;
mod census;

#[tokio::main]
async fn main() {
    println!("Conducting census...");

    let root_url = "http://localhost:8080"; // todo read from somewhere
    let conductor = User::new("admin@dsh.ca.gov"); // todo read from somewhere

    let mut http_client = HttpClient::new(root_url);
    http_client.authenticate_as(&conductor)
        .await
        .expect("should be able to authenticate");

    let external = ExternalHospitalDataProvider::new(http_client);
    let census_service = CensusService::new(Box::new(external));

    let result = census_service.conduct_census().await;
    match result {
        Ok(census) => println!("Done with census! Result: \n{}", census),
        Err(error) => println!("Failed to conduct census: {:#?}", error)
    };
}
