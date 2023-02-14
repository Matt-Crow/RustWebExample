// routes connect HTTP verbs & URLs to backend services

use std::collections::HashSet;

use actix_web::{web::{ServiceConfig, resource, get, Json, self, post, delete}, error::{ErrorInternalServerError, ErrorNotFound, ErrorBadRequest}, Responder, HttpResponse};
use serde::Deserialize;
use tokio::sync::Mutex;

use crate::{core::hospital_services::HospitalService, patient_services::{PatientService, PatientError}};
use common::hospital::{Hospital, Patient, AdmissionStatus};

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
    cfg.service(
        resource("/patients")
            .name("patients")
            .route(get().to(patients_get_handler))
    );
    cfg.service(
        resource("/waitlist")
            .name("waitlist")
            .route(post().to(waitlist_post_handler))  
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

#[derive(Debug, Deserialize)]
struct NewPatient {
    name: String,
    disallow_admission_to: Option<HashSet<String>>
}

async fn admit_patient(
    hospitals: web::Data<Mutex<HospitalService>>,
    hospital_name: web::Path<String>,
    posted: Json<NewPatient>
) -> actix_web::Result<Json<Hospital>> {
    let mut updater = hospitals.lock().await;

    let mut patient = Patient::new(&posted.name)
        .with_random_id();
    if let Some(ref disallowed_hospitals) = posted.disallow_admission_to {
        patient = patient.with_disallowed_hospitals(disallowed_hospitals);
    }
    println!("Patient: {:#?}", patient);

    match updater.admit_patient_to_hospital(patient, &hospital_name).await {
        Ok(hospital) => Ok(Json(hospital)),
        Err(e) => Err(ErrorInternalServerError(e))
    }
}

async fn unadmit_patient(
    hospitals: web::Data<Mutex<HospitalService>>,
    path: web::Path<(String, uuid::Uuid)>,
) -> impl Responder {
    let mut deleter = hospitals.lock().await;
    let hospital_name = &path.0;
    let patient_id = path.1;

    match deleter.unadmit_patient_from_hospital(patient_id, hospital_name).await {
        Ok(_) => Ok(HttpResponse::NoContent().body("")),
        Err(e) => Err(ErrorInternalServerError(e))
    }
}

async fn patients_get_handler(
    patients: web::Data<Mutex<PatientService>>
) -> impl Responder {
    let mut service = patients.lock().await;

    service.get_all_patients()
        .await
        .map(|ps| HttpResponse::Ok().json(ps))
        .map_err(ErrorInternalServerError)
}

/// handles POST requests to add a new patient to the waitlist
async fn waitlist_post_handler(
    patients: web::Data<Mutex<PatientService>>,
    posted: Json<NewPatient>
) -> impl Responder {
    let mut service = patients.lock().await;

    let mut patient = Patient::new(&posted.name)
        .with_status(AdmissionStatus::New);

    if let Some(ref disallowed_hospitals) = posted.disallow_admission_to {
        patient = patient.with_disallowed_hospitals(disallowed_hospitals);
    }
    println!("Patient: {:#?}", patient);

    match service.add_patient_to_waitlist(&patient).await {
        Ok(stored) => Ok(HttpResponse::Created().json(stored)),
        Err(e) => match e {
            PatientError::AlreadyExists(id) => Err(ErrorBadRequest(format!("patient with ID {} already exists", id))),
            PatientError::Repository(inner) => Err(ErrorInternalServerError(inner))
        }
    }
}