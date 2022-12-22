use actix_web::{web::{self, Json}, Responder, HttpResponse, error};

use crate::{
    models::anchor::Anchor, 
    services::service_provider::ServiceProvider
};

pub fn configure_anchor_controller_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/anchors")
        .name("news_anchor")
        .route(web::get().to(get_all_anchors))
        .route(web::post().to(post_anchor))
    );
    cfg.service(web::resource("/anchors/{name}")
        .name("news_anchor")
        .route(web::get().to(get_anchor_by_name))
        .route(web::put().to(put_anchor))
        .route(web::delete().to(delete_anchor))
    );
}

async fn get_all_anchors(service_provider: web::Data<ServiceProvider>) -> actix_web::Result<Json<Vec<Anchor>>> {
    let lock = service_provider.anchors();
    let anchors = lock.lock().unwrap();

    match anchors.get_all() {
        Ok(data) => Ok(Json(data)),
        Err(msg) => Err(error::ErrorInternalServerError(msg)) // use error helper functions to create actix errors
    }
}

async fn get_anchor_by_name(name: web::Path<String>) -> Json<Anchor> {
    Json(Anchor::new(&name))
}

async fn post_anchor(anchor: Json<Anchor>, service_provider: web::Data<ServiceProvider>) -> actix_web::Result<Json<Anchor>> {
    let lock = service_provider.anchors();
    let mut creator = lock.lock().unwrap();
    let a: Anchor = anchor.0;
    match creator.create(a) {
        Ok(data) => Ok(Json(data)),
        Err(msg) => Err(error::ErrorBadRequest(msg))
    }
}

async fn put_anchor(name: web::Path<String>, anchor: Json<Anchor>) -> Json<Anchor> {
    let old = Anchor::new(&name.into_inner())
        .with_id(12345)
        .with_accuracy(0.5)
        .with_years_employed(5);
    Json(Anchor::merge(&old, &anchor.into_inner()))
}

async fn delete_anchor(name: web::Path<String>) -> impl Responder {
    println!("TODO delete {}", name);
    HttpResponse::NoContent()
}