// Declare which modules (folders) should be compiled / loaded.
// These are searched recursively to load any of their declared modules as well.
pub mod core; // can declare modules as public in case other programs need them
mod infrastructure;

use actix_web::{
    HttpServer, 
    App,
    web
};
use crate::{
    core::services::service_provider::ServiceProvider,
    infrastructure::anchor_routes::configure_anchor_routes
};

#[actix_web::main]
async fn main() -> std::io::Result<()> { // "()" is essentially "null"

    // The Rust ecosystem does not appear to have a good Dependency Injection
    // framework, so we have to bundle together the service providers ourselves.
    let sp = web::Data::new(ServiceProvider::default());

    println!("Starting web server...");
    
    HttpServer::new(move || {
        App::new()
            .app_data(sp.clone()) // app data is thread-safe
            .service(web::scope("/api/v1") // register API routes
                .configure(configure_anchor_routes)
            )
        })
        .bind(("127.0.0.1", 8080))? // "?" means "return error if this fails, else unwrap"
        .run()
        .await
}
