pub mod controllers;
pub mod models;
pub mod repositories;
pub mod services;

use actix_web::{
    Responder, HttpServer, App, get, web
};
use crate::controllers::{forecast_controller::configure_forecast_controller_routes, anchor_controller::configure_anchor_controller_routes};

// "impl Responder" means, "can be converted to HTTP response"
#[get("/")] // trait-based routing
async fn index() -> impl Responder {
    "This is the main page of the website."
} // todo try some front-end libraries here

#[actix_web::main]
async fn main() -> std::io::Result<()> { // "()" is essentially "null"
    println!("Starting web server...");

    HttpServer::new(|| {
        App::new()
            .configure(configure_forecast_controller_routes)
            .service(index) // Use service to register routes decorated with macros
            .service(web::scope("/api/v1")
                .configure(configure_forecast_controller_routes)
                .configure(configure_anchor_controller_routes)
            )
    })
    .bind(("127.0.0.1", 8080))? // "?" means "return error if this fails, else unwrap"
    // todo print message once the server starts
    .run()
    .await
}
