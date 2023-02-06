use actix_web::{web::{ServiceConfig, Json, self, post}, HttpResponse};
use serde::Deserialize;

use crate::core::{service_provider::ServiceProvider, users::User};

pub fn configure_authentication_routes(cfg: &mut ServiceConfig) {
    cfg.route("/signup", post().to(handle_signup)); // rm this?
}

#[derive(Deserialize)]
struct SignupRequest {
    username: String
}

async fn handle_signup(
    signup_request: Json<SignupRequest>,
    service_provider: web::Data<ServiceProvider>
) -> actix_web::Result<HttpResponse> {

    let mut users = service_provider.users().lock().await;

    let created = users.create(&User::new(&signup_request.username))
        .await
        .expect("msg");

    Ok(HttpResponse::Created().json(created))
}