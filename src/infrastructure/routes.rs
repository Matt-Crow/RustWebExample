// routes connect HTTP verbs & URLs to backend services

use actix_web::{web::{ServiceConfig, resource, get, Json, self}, error::ErrorInternalServerError};

use crate::core::{hospital_models::Hospital, services::service_provider::ServiceProvider};

pub fn configure_hospital_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        resource("/hospitals")
            .name("hospitals")
            .route(get().to(get_all_hospitals))
    );
}

async fn get_all_hospitals(
    services: web::Data<ServiceProvider>
) -> actix_web::Result<Json<Vec<Hospital>>> {
    let mutex = services.hospitals();
    let getter = mutex.lock().unwrap();

    Err(ErrorInternalServerError("err"))
}