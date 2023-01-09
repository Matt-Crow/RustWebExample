// Since Rust is not an object-oriented language, and Actix Web is a Rust
// framework, it does not use the .NET style of controller classes.
// Instead, Actix uses request handler functions that are registered with the
// ServiceConfig.

use actix_web::{
    web::{
        self, 
        Json
    }, 
    error
};

use crate::core::{
    models::anchor::Anchor, 
    services::service_provider::ServiceProvider
};

/// Registers the various anchor-related routes, allowing the app to handler
/// requests to these endpoints.
pub fn configure_anchor_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/anchors/{id}")
            .name("news_anchor")
            .route(web::put().to(put_anchor))
    );
}

async fn put_anchor(_id: web::Path<u32>, anchor: Json<Anchor>, service_provider: web::Data<ServiceProvider>) -> actix_web::Result<Json<Anchor>> {
    let mutex = service_provider.anchors();
    let mut updater = mutex.lock().unwrap();

    match updater.update(&anchor) {
        Ok(data) => Ok(Json(data)),
        Err(msg) => Err(error::ErrorBadRequest(msg))
    }
}