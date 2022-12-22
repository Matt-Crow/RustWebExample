// Declare which modules (folders) should be compiled / loaded.
// These are searched recursively to load any of their declared modules as well.
mod controllers;
pub mod models; // makes the models module upblic, in case other programs need it
mod repositories;
mod services;

use actix_web::{
    Responder, // "this can be converted to an HTTP response"
    HttpServer, 
    App, 
    get, 
    web
};
use crate::{
    controllers::{
        forecast_controller::configure_forecast_controller_routes, 
        anchor_controller::configure_anchor_controller_routes
    }, 
    services::service_provider::ServiceProvider
};

#[get("/")] // trait-based routing
async fn index() -> impl Responder {
    "This is the main page of the website."
} // todo try some front-end libraries here

#[actix_web::main]
async fn main() -> std::io::Result<()> { // "()" is essentially "null"
    println!("Starting web server...");

    let sp = web::Data::new(ServiceProvider::default());

    HttpServer::new(move || {
        App::new()
            .app_data(sp.clone())
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
