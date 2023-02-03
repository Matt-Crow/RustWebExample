// Declare which modules (folders) should be compiled / loaded.
// These are searched recursively to load any of their declared modules as well.
pub mod core; // can declare modules as public in case other programs need them
mod infrastructure;

use std::env;

use actix_session::{SessionMiddleware, storage::CookieSessionStore};
use actix_web::{HttpServer, App, web, cookie::Key};
use actix_web_httpauth::middleware::HttpAuthentication;
use crate::{
    core::service_provider::ServiceProvider,
    infrastructure::{routes::configure_hospital_routes, authentication::{jwt::{jwt_auth_middleware, configure_jwt_routes}, routes::configure_authentication_routes, openid::{OpenIdService, configure_openid_routes}}, database::{database_hospital_repository::DatabaseHospitalRepository, database_user_repository::DatabaseUserRepository, pool::make_db_pool}}
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    let openid_service = OpenIdService::from_env()
        .await
        .expect("Should be able to start OpenID service");

    let pool = make_db_pool()
        .await
        .expect("Database pool should initialize successfully");
    
    let mut hospital_repo = DatabaseHospitalRepository::new(pool.clone());
    let user_repo = DatabaseUserRepository::new(pool.clone());

    let args: Vec<String> = env::args().collect();
    if args.iter().any(|arg| arg == "--setup") {
        let r = hospital_repo.setup().await;
        println!("Setup result: {:#?}", r);
    }

    // The Rust ecosystem does not appear to have a good Dependency Injection
    // framework, so we have to bundle together the service providers ourselves.
    let sp = web::Data::new(ServiceProvider::new(hospital_repo, user_repo));
    let oid = web::Data::new(openid_service);    

    println!("Starting web server...");
    
    HttpServer::new(move || {
        App::new()
            .app_data(sp.clone()) // app data is thread-safe
            .app_data(oid.clone())
            // the session allows us to persist data across requests and associate
            // it with a single user. This demo uses a cookie to store all the
            // session data, as the Actix Session package does not support storing
            // in MSSQL. Ideally, we would store a key in the user's cookies, and
            // their associated data in MSSQL
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                Key::from("super-secret-key-that-must-be-at-least-64-bytes-long-so-I-guess-I-will-just-have-to-make-something-up".as_bytes())
            ))
            .configure(configure_authentication_routes)
            .configure(configure_jwt_routes)
            .configure(configure_openid_routes)
            .service(web::scope("/api/v1") // register API routes
                .wrap(HttpAuthentication::bearer(jwt_auth_middleware))
                .configure(configure_hospital_routes)
            )
        })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
