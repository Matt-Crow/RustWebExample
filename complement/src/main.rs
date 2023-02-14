use std::{collections::HashSet, sync::Arc};

use actix_web::{get, Responder, HttpServer, App, HttpResponse, web::{Json, self}, error::ErrorInternalServerError};
use async_trait::async_trait;
use common::hospital::HospitalNameProvider;
use serde::{Deserialize, Serialize};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting complement service on localhost:8081");
    HttpServer::new(|| {
        App::new()
            .app_data(web::Data::new(RemoteHospitalNameProvider::new("http://localhost:8080/api/v1/hospital-names")))
            .service(complement_handler)
        })
        .bind(("127.0.0.1", 8081))?
        .run()
        .await
}

/// makes requests to another service to get the list of hospital names
struct RemoteHospitalNameProvider {
    url: String
}

impl RemoteHospitalNameProvider {
    fn new(url: &str) -> Self {
        Self {
            url: String::from(url)
        }
    }
}

#[async_trait]
impl HospitalNameProvider for RemoteHospitalNameProvider {
    async fn get_all_hospital_names(&self) -> Result<Vec<String>, common::hospital::Error> {
        let r: HospitalNames = reqwest::get(&self.url)
            .await
            .map_err(common::hospital::Error::external_service_error)?
            .json()
            .await
            .map_err(common::hospital::Error::external_service_error)?;
        Ok(r.names.into_iter().collect())
    }
}

/// handles GET request to localhost:8081/complement, and responds with the
/// complement of the body
#[get("/complement")]
async fn complement_handler(
    name_provider: web::Data<RemoteHospitalNameProvider>,
    complement_me: Json<HospitalNames>
) -> impl Responder {
    complement_hospitals(name_provider.into_inner(), &complement_me.names)
        .await
        .map(|r| HttpResponse::Ok().json(HospitalNames::new(r)))
        .map_err(|e| ErrorInternalServerError(e))
}

/// "nicer" than responding with just a JSON array
#[derive(Debug, Deserialize, Serialize)]
struct HospitalNames {
    names: HashSet<String>
}

impl HospitalNames {
    fn new(names: HashSet<String>) -> Self {
        Self {
            names
        }
    }
}

/// queries another service for hospital names, uses those as the universal set,
/// then returns the complement of the input
async fn complement_hospitals(
    name_provider: Arc<RemoteHospitalNameProvider>,
    hospitals: &HashSet<String>)
-> Result<HashSet<String>, common::hospital::Error> {
    let u = name_provider.get_all_hospital_names()
        .await?
        .into_iter()
        .collect();
    Ok(complement_of(hospitals, &u))
}

/// given (s, u), computes u - s (A.K.A. s')
fn complement_of(s: &HashSet<String>, universal_set: &HashSet<String>) -> HashSet<String> {
    universal_set.difference(&s)
        .map(String::from)
        .collect()
}