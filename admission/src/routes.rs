// routes connect HTTP verbs & URLs to backend services

use std::collections::HashSet;

use actix_web::{web::{ServiceConfig, resource, get, Json, self, post, delete}, error::{ErrorInternalServerError, ErrorNotFound, ErrorBadRequest}, Responder, HttpResponse};
use serde::Deserialize;
use tokio::sync::Mutex;

use crate::{hospital_services::HospitalService, patient_services::{PatientService, PatientError}};
use common::{patient::Patient, hospital::{Hospital, GetHospitalNamesResponse}};

/// sets up routing
pub fn configure_hospital_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        resource("/hospital-names")
            .name("hospital names")
            .route(get().to(get_hospital_names_handler))
    );
    cfg.service(
        resource("/hospitals")
            .name("hospitals")
            .route(get().to(get_all_hospitals))
    );
    cfg.service(
        resource("/hospitals/admit-from-waitlist")
            .name("admit from waitlist")
            .route(post().to(post_admit_from_waitlist_handler))
    );
    cfg.service(
        resource("/hospitals/{name}")
            .name("hospital")
            .route(get().to(get_hospital_by_name))
    );
    cfg.service(
        resource("/hospitals/{name}/{patient_id}")
            .name("hospital_patients")
            .route(delete().to(unadmit_patient))
    );
    cfg.service(
        resource("/waitlist")
            .name("waitlist")
            .route(get().to(waitlist_get_handler))
            .route(post().to(waitlist_post_handler))  
    );
}

/// handles requests to GET /hospital-names
/// must be async to work with actix
async fn get_hospital_names_handler(
    // web::Data grabs shared state registered in main.rs
    // register using web::Data<T>
    // grab using web::Data<T>
    // This is one way of doing dependency injection
    hospitals: web::Data<Mutex<HospitalService>>

    // routing functions can return a lot of different things, not just this
    // however, given how errors propogate up from lower layers, they usually
    // need to return a Result to account for the possibility of errors
    // actix web has its own Result type, not to be confused with Rust's
) -> actix_web::Result<Json<GetHospitalNamesResponse>> {
    
    // Rust is really picky about shared concurrency - if we want to modify a
    // shared resource, it must be wrapped in a mutex.
    // Note that although getting is supposed to be a non-mutable operation,
    // the various Tiberius query methods all require mutablity for some reason,
    // so that mutablity also propogates upward.
    let mut getter = hospitals.lock().await;

    getter.get_all_hospitals()
        .await
        .map(|hospitals| {
            let names: HashSet<String> = hospitals.iter()
                .map(|h| h.name())
                .collect();
            Json(GetHospitalNamesResponse::new(&names))
        })
        .map_err(ErrorInternalServerError)
}

async fn get_all_hospitals(
    hospitals: web::Data<Mutex<HospitalService>>
) -> actix_web::Result<Json<Vec<Hospital>>> {
    
    let mut getter = hospitals.lock().await;

    match getter.get_all_hospitals().await {
        Ok(hospitals) => Ok(Json(hospitals)),
        Err(error) => Err(ErrorInternalServerError(error))
    }
}

async fn post_admit_from_waitlist_handler(
    patients: web::Data<Mutex<PatientService>>
) -> actix_web::Result<Json<Vec<Patient>>> {

    let mut admitter = patients.lock().await;

    admitter.admit_patients_from_waitlist()
        .await 
        .map(Json)
        .map_err(ErrorInternalServerError)
}

async fn get_hospital_by_name(
    hospitals: web::Data<Mutex<HospitalService>>, // grab shared data
    name: web::Path<String> // grab from URL path
) -> actix_web::Result<Json<Hospital>> { // return as JSON

    let mut getter = hospitals.lock().await;

    getter.get_hospital_by_name(&name).await
        .map_err(ErrorInternalServerError)? // 500 error if getter fails
        .map(Json)                          // 200 if found
        .ok_or_else(|| ErrorNotFound(name)) // 404 if not found
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

async fn waitlist_get_handler(
    patients: web::Data<Mutex<PatientService>>
) -> impl Responder {
    let mut service = patients.lock().await;

    service.get_waitlisted_patients()
        .await
        .map(|ps| HttpResponse::Ok().json(ps))
        .map_err(ErrorInternalServerError)
}

// Can't use the full patient struct, as then the poster could provide the
// patient ID or other details, which we don't want. It's oftentimes helpful to
// create structures such as this that only contain a subset of another struct's
// fields.
#[derive(Debug, Deserialize)]
#[serde(rename_all="camelCase")]
struct NewPatientRequest {
    name: String,
    disallow_admission_to: Option<HashSet<String>>
}

/// handles POST requests to add a new patient to the waitlist
async fn waitlist_post_handler(
    patients: web::Data<Mutex<PatientService>>,
    posted: Json<NewPatientRequest>
) -> impl Responder {
    
    let mut patient = Patient::new(&posted.name);

    if let Some(ref disallowed_hospitals) = posted.disallow_admission_to {
        patient = patient.with_disallowed_hospitals(disallowed_hospitals);
    }
    println!("Patient: {:#?}", patient);

    // Wait as long as possible before locking - this minimizes the chance of
    // this request blocking another. (keep a small Critical Section)
    let mut waitlister = patients.lock().await;

    match waitlister.add_patient_to_waitlist(&patient).await {
        Ok(stored) => Ok(HttpResponse::Created().json(stored)),
        Err(e) => match e {
            PatientError::AlreadyExists(id) => Err(ErrorBadRequest(format!("patient with ID {} already exists", id))),
            PatientError::Repository(inner) => Err(ErrorInternalServerError(inner)),
            PatientError::Unsupported => Err(ErrorInternalServerError("?"))
        }
    }
}