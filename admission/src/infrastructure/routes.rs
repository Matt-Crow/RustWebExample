// routes connect HTTP verbs & URLs to backend services

use actix_web::{web::{ServiceConfig, resource, get, Json, self, post, delete}, error::{ErrorInternalServerError, ErrorNotFound}, Responder, HttpResponse};
use tokio::sync::Mutex;

use crate::core::hospital_services::HospitalService;
use common::hospital::{Hospital, Patient};

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
    cfg.service(
        resource("/hospitals/{name}/{patient_id}")
            .name("hospital_patients")
            .route(delete().to(unadmit_patient))
    );
}

async fn get_all_hospitals(
    // web::Data grabs shared state registered during app creation
    hospitals: web::Data<Mutex<HospitalService>>
) -> actix_web::Result<Json<Vec<Hospital>>> {
    // actix web has its own Result type, not to be confused with Rust's
    // since the app state is shared across threads, need mutex to use it
    let mut mutex = hospitals.lock().await;

    match mutex.get_all_hospitals().await {
        Ok(hospitals) => Ok(Json(hospitals)),
        Err(error) => Err(ErrorInternalServerError(error))
    }
}

async fn get_hospital_by_name(
    hospitals: web::Data<Mutex<HospitalService>>,
    name: web::Path<String>
) -> actix_web::Result<Json<Hospital>> {
    let mut getter = hospitals.lock().await;

    match getter.get_hospital_by_name(&name).await {
        Ok(maybe_hospital) => match maybe_hospital {
            Some(hospital) => Ok(Json(hospital)),
            None => Err(ErrorNotFound(format!("Invalid hospital name: {}", name)))        
        },
        Err(e) => {
            Err(ErrorInternalServerError(e))
        }
    }
}

async fn admit_patient(
    hospitals: web::Data<Mutex<HospitalService>>,
    hospital_name: web::Path<String>,
    patient: Json<Patient>
) -> actix_web::Result<Json<Hospital>> {
    let mut updater = hospitals.lock().await;

    match updater.admit_patient_to_hospital(patient.0, &hospital_name).await {
        Ok(hospital) => Ok(Json(hospital)),
        Err(e) => Err(ErrorInternalServerError(e))
    }
}

async fn unadmit_patient(
    hospitals: web::Data<Mutex<HospitalService>>,
    path: web::Path<(String, u32)>,
) -> impl Responder {
    let mut deleter = hospitals.lock().await;
    let hospital_name = &path.0;
    let patient_id = path.1;

    match deleter.unadmit_patient_from_hospital(patient_id, hospital_name).await {
        Ok(_) => Ok(HttpResponse::NoContent().body("")),
        Err(e) => Err(ErrorInternalServerError(e))
    }
}