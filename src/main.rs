// Declare which modules (folders) should be compiled / loaded.
// These are searched recursively to load any of their declared modules as well.
mod core; // can declare modules as public in case other programs need them
mod infrastructure;

use std::env;

use actix_web::{HttpServer, App, web};
use actix_web_httpauth::middleware::HttpAuthentication;
use crate::{
    core::service_provider::ServiceProvider,
    infrastructure::{routes::configure_hospital_routes, authentication::jwt::{jwt_auth_middleware, configure_jwt_routes}, database::{connection::create_client_from_env, database_repository::DatabaseHospitalRepository}}
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mssql_client = create_client_from_env()
        .await
        .expect("Failed to create mssql client!");
    
    let mut repo = DatabaseHospitalRepository::new(mssql_client);

    let args: Vec<String> = env::args().collect();
    if args.iter().any(|arg| arg == "--setup") {
        let r = repo.setup().await;
        println!("Setup result: {:#?}", r);
    }
    /*
    let db_connection_pool = make_db_pool().await;
    println!("DB connection pool: {:#?}", db_connection_pool);
    let c = db_connection_pool.unwrap();
    let conn = c.get().await; // times out here
    println!("Connection: {:#?}", conn);
    */

    // The Rust ecosystem does not appear to have a good Dependency Injection
    // framework, so we have to bundle together the service providers ourselves.
    let sp = web::Data::new(ServiceProvider::default());

    println!("Starting web server...");
    
    HttpServer::new(move || {
        App::new()
            .app_data(sp.clone()) // app data is thread-safe
            .configure(configure_jwt_routes)
            .service(web::scope("/api/v1") // register API routes
                .wrap(HttpAuthentication::bearer(jwt_auth_middleware))
                .configure(configure_hospital_routes)
            )
        })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
