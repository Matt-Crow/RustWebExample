use std::{collections::HashSet, sync::Arc};

use actix_web::{get, Responder, HttpServer, App, HttpResponse, web::{Json, self}, error::ErrorInternalServerError};
use async_trait::async_trait;
use common::{hospital::{GetHospitalNamesResponse, GetHospitalNames, HospitalError, GetHospitalNamesRequest}, user::User, http_client::HttpClient};
use tokio::sync::Mutex;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut client = HttpClient::new("http://localhost:8080"); // todo read URL from env
    let user = User::new("Complement Demo");
    client.authenticate_as(&user)
        .await
        .expect("should be able to authenticate");
    let shared_state = web::Data::new(Mutex::new(RemoteHospitalNameProvider::new(client)));

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
impl GetHospitalNames for RemoteHospitalNameProvider {
    async fn get_hospital_names(&mut self, _request: GetHospitalNamesRequest) -> Result<GetHospitalNamesResponse, HospitalError> {
        let r: GetHospitalNamesResponse = self.client.get("/api/v1/hospital-names")
            .await
            .map_err(HospitalError::external_service_error)?
            .json()
            .await
            .map_err(HospitalError::external_service_error)?;
        Ok(r)
    }
}

/// handles GET request to localhost:8081/complement, and responds with the
/// complement of the body
#[get("/complement")]
async fn complement_handler(
    name_provider: web::Data<Mutex<RemoteHospitalNameProvider>>,
    complement_me: Json<GetHospitalNamesResponse>
) -> impl Responder {
    complement_hospitals(name_provider.into_inner(), &complement_me.hospital_names())
        .await
        .map(|r| HttpResponse::Ok().json(GetHospitalNamesResponse::new(&r)))
        .map_err(ErrorInternalServerError)
}

/// queries another service for hospital names, uses those as the universal set,
/// then returns the complement of the input
async fn complement_hospitals(
    name_provider: Arc<Mutex<RemoteHospitalNameProvider>>,
    hospitals: &HashSet<String>)
-> Result<HashSet<String>, HospitalError> {
    let mut lock = name_provider.lock().await;
    let u = lock.get_hospital_names(GetHospitalNamesRequest::new())
        .await?
        .hospital_names();
    let complement = u.difference(hospitals) // A' = U - A
        .map(String::from) // Iterator<&str> -> Iterator<String>
        .collect();        // Iterator<String> -> HashSet<String>
    Ok(complement)
}