pub mod controllers;
pub mod services;

use actix_web::{
    Responder, HttpResponse, HttpServer, App, get
};
use crate::controllers::forecast_controller::configure_forecast_controller_routes;

// "impl Responder" means, "can be converted to HTTP response"
#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body("This is the main page of the website.")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting web server...");

    HttpServer::new(|| {
        App::new()
            .configure(configure_forecast_controller_routes)
            .service(index) // Use service to register routes decorated with macros
    })
    .bind(("127.0.0.1", 8080))? // "?" means "return error if this fails, else unwrap"
    .run()
    .await
}
