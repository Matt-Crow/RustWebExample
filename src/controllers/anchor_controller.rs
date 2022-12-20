use actix_web::web::{self, Json};

use crate::models::anchor::Anchor;

pub fn configure_anchor_controller_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/anchors/{name}")
        .name("news_anchor")
        .route(web::get().to(get_anchor_by_name))
        .route(web::post().to(post_anchor))
    );
}

async fn get_anchor_by_name(name: web::Path<String>) -> Json<Anchor> {
    Json(Anchor::new(&name))
}

async fn post_anchor(anchor: Json<Anchor>) -> Json<Anchor> {
    Json(anchor.with_id(12345))
}