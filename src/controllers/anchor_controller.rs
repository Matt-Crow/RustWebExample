use actix_web::{web::{self, Json}, Responder, HttpResponse};

use crate::models::anchor::Anchor;

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

async fn get_all_anchors() -> Json<Vec<Anchor>> {
    Json(vec![
        Anchor::new("Foo"),
        Anchor::new("Bar")
            .with_years_employed(2),
        Anchor::new("Baz")
            .with_years_employed(5)
            .with_accuracy(0.33)
    ])
}

async fn get_anchor_by_name(name: web::Path<String>) -> Json<Anchor> {
    Json(Anchor::new(&name))
}

async fn post_anchor(anchor: Json<Anchor>) -> Json<Anchor> {
    Json(anchor.with_id(12345))
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