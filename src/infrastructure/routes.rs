// routes connect HTTP verbs & URLs to backend services

use actix_web::{web::{ServiceConfig, resource, get, Json, self, post}, error::{ErrorInternalServerError, ErrorNotFound}};

use crate::core::{hospital_models::{Hospital, Patient}, services::service_provider::ServiceProvider};

pub fn configure_hospital_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        resource("/hospitals")
            .name("hospitals")
            .route(get().to(get_all_hospitals))
    );
    cfg.service(
        resource("/hospitals/{name}")
            .name("hospital")
            .route(get().to(get_hospital_by_name))
            .route(post().to(admit_patient))
    );
}

// todo only auth users can see patients
async fn get_all_hospitals(
    // web::Data grabs shared state registered during app creation
    services: web::Data<ServiceProvider>
) -> actix_web::Result<Json<Vec<Hospital>>> {
    // actix web has its own Result type, not to be confused with Rust's
    // since the app state is shared across threads, need mutex to use it
    let mutex = services.hospitals();
    let getter = mutex.lock().unwrap();

    match getter.get_all_hospitals() {
        Ok(hospitals) => Ok(Json(hospitals)),
        Err(error) => Err(ErrorInternalServerError(error))
    }
}

// todo only auth users can view patients
async fn get_hospital_by_name(
    services: web::Data<ServiceProvider>,
    name: web::Path<String>
) -> actix_web::Result<Json<Hospital>> {
    let mutex = services.hospitals().lock();
    let getter = mutex.unwrap();

    match getter.get_hospital_by_name(&name) {
        Ok(maybe_hospital) => match maybe_hospital {
            Some(hospital) => Ok(Json(hospital)),
            None => Err(ErrorNotFound(format!("Invalid hospital name: {}", name)))        
        },
        Err(e) => {
            Err(ErrorInternalServerError(e))
        }
    }
}

// todo only auth users
async fn admit_patient(
    services: web::Data<ServiceProvider>,
    hospital_name: web::Path<String>,
    patient: Json<Patient>
) -> actix_web::Result<Json<Hospital>> {
    let mutex = services.hospitals().lock();
    let mut updater = mutex.unwrap();

    match updater.admit_patient_to_hospital(patient.0, &hospital_name) {
        Ok(hospital) => Ok(Json(hospital)),
        Err(e) => Err(ErrorInternalServerError(e))
    }
}