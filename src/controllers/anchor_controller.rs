// Since Rust is not an object-oriented language, and Actix Web is a Rust
// framework, it does not use the .NET style of controller classes.
// Instead, Actix uses request handler functions that are registered with the
// ServiceConfig.

use actix_web::{
    web::{
        self, 
        Json
    }, 
    Responder, 
    HttpResponse, 
    error
};

use crate::{
    models::anchor::Anchor, 
    services::service_provider::ServiceProvider
};

/// Registers the various anchor-related routes, allowing the app to handler
/// requests to these endpoints.
pub fn configure_anchor_controller_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/anchors")
            .name("news_anchor")
            .route(web::get().to(get_all_anchors))
            .route(web::post().to(post_anchor))
    );
    cfg.service(
        web::resource("/anchors/{id}")
            .name("news_anchor")
            .route(web::get().to(get_anchor_by_id))
            .route(web::put().to(put_anchor))
            .route(web::delete().to(delete_anchor))
    );
}

async fn get_all_anchors(
    // web::Data<T> grabs shared state registered during app creation
    service_provider: web::Data<ServiceProvider>
) -> actix_web::Result<Json<Vec<Anchor>>> {
    // Actix Web has its own Result<T, E> type, not to be confused with Rust's

    // since the app state is shared across threads, we must grab the mutex to
    // use it
    let mutex = service_provider.anchors();
    let anchors = mutex.lock().unwrap();

    match anchors.get_all() {
        Ok(data) => Ok(Json(data)),
        Err(msg) => Err(error::ErrorInternalServerError(msg)) // use error helper functions to create actix errors
    }
}

async fn get_anchor_by_id(
    id: web::Path<u32>, // grab path variables
    service_provider: web::Data<ServiceProvider>
) -> actix_web::Result<Json<Anchor>> {
    let mutex = service_provider.anchors();
    let getter = mutex.lock().unwrap();
    match getter.get_by_id(*id) {
        Ok(maybe_anchor) => match maybe_anchor {
            Some(anchor) => Ok(Json(anchor)),
            None => Err(error::ErrorNotFound(format!("No anchor found with ID = {}", id)))
        },
        Err(msg) => Err(error::ErrorBadRequest(msg))
    }
}

async fn post_anchor(anchor: Json<Anchor>, service_provider: web::Data<ServiceProvider>) -> actix_web::Result<Json<Anchor>> {
    let mutex = service_provider.anchors();
    let mut creator = mutex.lock().unwrap();
    match creator.create(&anchor) {
        Ok(data) => Ok(Json(data)),
        Err(msg) => Err(error::ErrorBadRequest(msg))
    }
}

async fn put_anchor(_id: web::Path<u32>, anchor: Json<Anchor>, service_provider: web::Data<ServiceProvider>) -> actix_web::Result<Json<Anchor>> {
    let mutex = service_provider.anchors();
    let mut updater = mutex.lock().unwrap();
    match updater.update(&anchor) {
        Ok(data) => Ok(Json(data)),
        Err(msg) => Err(error::ErrorBadRequest(msg))
    }
}

async fn delete_anchor(id: web::Path<u32>, service_provider: web::Data<ServiceProvider>) -> impl Responder {
    let mutex = service_provider.anchors();
    let mut deletor = mutex.lock().unwrap();
    match deletor.delete_anchor(*id) {
        Ok(_) => HttpResponse::NoContent().body(""),
        Err(msg) => HttpResponse::BadRequest().body(msg)
    }
}