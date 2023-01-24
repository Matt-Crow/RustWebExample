// Declare which modules (folders) should be compiled / loaded.
// These are searched recursively to load any of their declared modules as well.
pub mod core; // can declare modules as public in case other programs need them
mod infrastructure;

use std::{env, sync::Arc};

use actix_web::{HttpServer, App, web};
use actix_web_httpauth::middleware::HttpAuthentication;
use tokio::sync::Mutex;
use crate::{
    core::service_provider::ServiceProvider,
    infrastructure::{routes::configure_hospital_routes, authentication::{jwt::{jwt_auth_middleware, configure_jwt_routes}, routes::configure_authentication_routes}, database::{connection::create_client_from_env, database_hospital_repository::DatabaseHospitalRepository, database_user_repository::DatabaseUserRepository}}
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mssql_client = Arc::new(Mutex::new(create_client_from_env()
        .await
        .expect("Failed to create mssql client!")));
    
    let mut hospital_repo = DatabaseHospitalRepository::new(mssql_client.clone());
    let user_repo = DatabaseUserRepository::new(mssql_client.clone());

    let args: Vec<String> = env::args().collect();
    if args.iter().any(|arg| arg == "--setup") {
        let r = hospital_repo.setup().await;
        println!("Setup result: {:#?}", r);
    }

    // The Rust ecosystem does not appear to have a good Dependency Injection
    // framework, so we have to bundle together the service providers ourselves.
    let sp = web::Data::new(ServiceProvider::new(hospital_repo, user_repo));    

    println!("Starting web server...");
    
    HttpServer::new(move || {
        App::new()
            .app_data(sp.clone()) // app data is thread-safe
            .configure(configure_authentication_routes)
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
