use std::{collections::HashSet, sync::Arc};

use actix_web::{get, Responder, HttpServer, App, HttpResponse, web::{Json, self}, error::ErrorInternalServerError};
use async_trait::async_trait;
use common::{hospital::HospitalNameProvider, user::User, http_client::HttpClient, hospital_names::HospitalNames};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut client = HttpClient::new("http://localhost:8080"); // todo read URL from env
    let user = User::new("Complement Demo");
    client.authenticate_as(&user)
        .await
        .expect("should be able to authenticate");
    let shared_state = web::Data::new(RemoteHospitalNameProvider::new(client));

    println!("Starting complement service on localhost:8081");
    HttpServer::new(move || {
        App::new()
            .app_data(shared_state.clone())
            .service(complement_handler)
        })
        .bind(("127.0.0.1", 8081))?
        .run()
        .await
}

/// makes requests to another service to get the list of hospital names
struct RemoteHospitalNameProvider {
    client: HttpClient
}

impl RemoteHospitalNameProvider {
    fn new(client: HttpClient) -> Self {
        Self {
            client
        }
    }
}

#[async_trait]
impl HospitalNameProvider for RemoteHospitalNameProvider {
    async fn get_all_hospital_names(&self) -> Result<HospitalNames, common::hospital::Error> {
        let r: HospitalNames = self.client.get("/api/v1/hospital-names")
            .await
            .map_err(common::hospital::Error::external_service_error)?
            .json()
            .await
            .map_err(common::hospital::Error::external_service_error)?;
        Ok(r)
    }
}

/// handles GET request to localhost:8081/complement, and responds with the
/// complement of the body
#[get("/complement")]
async fn complement_handler(
    name_provider: web::Data<RemoteHospitalNameProvider>,
    complement_me: Json<HospitalNames>
) -> impl Responder {
    complement_hospitals(name_provider.into_inner(), &complement_me.names())
        .await
        .map(|r| HttpResponse::Ok().json(HospitalNames::new(r)))
        .map_err(ErrorInternalServerError)
}

/// queries another service for hospital names, uses those as the universal set,
/// then returns the complement of the input
async fn complement_hospitals(
    name_provider: Arc<RemoteHospitalNameProvider>,
    hospitals: &HashSet<String>)
-> Result<HashSet<String>, common::hospital::Error> {
    let u = name_provider.get_all_hospital_names()
        .await?
        .names();
    let complement = u.difference(hospitals) // A' = U - A
        .map(String::from) // Iterator<&str> -> Iterator<String>
        .collect();        // Iterator<String> -> HashSet<String>
    Ok(complement)
}