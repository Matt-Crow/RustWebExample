// Declare which modules (folders) should be compiled / loaded.
// These are searched recursively to load any of their declared modules as well.
mod authentication;
mod database;
mod hospital_services;
mod remote_complement_provider;
mod routes;
mod patient_services;
mod user_services;

use std::env;

use actix_session::{SessionMiddleware, storage::CookieSessionStore};
use actix_web::{HttpServer, App, web, cookie::Key};
use actix_web_httpauth::middleware::HttpAuthentication;
use common::complement_service::ComplementService;
use tokio::sync::Mutex;
use crate::{
    hospital_services::HospitalService,
    {routes::configure_hospital_routes, authentication::{jwt::{jwt_auth_middleware, configure_jwt_routes}, openid::{OpenIdService, configure_openid_routes}}, database::{database_hospital_repository::DatabaseHospitalRepository, pool::make_db_pool, database_group_repository::DatabaseGroupRepository, database_patient_repository::DatabasePatientRepository}}, patient_services::PatientService, remote_complement_provider::RemoteComplementProvider,
    user_services::UserService
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    let openid_service = OpenIdService::from_env()
        .await
        .expect("Should be able to start OpenID service");
    // expect causes the program to crash if the result is not OK.
    // Good for unrecoverable errors, or if an error occurs in a function that
    // doesn't return Result. While it may be tempting to slap this everywhere
    // so you don't have to propogate Results up the callers' return types,
    // that will make things a lot harder in the long run.

    let pool = make_db_pool()
        .await
        .expect("Database pool should initialize successfully");
    
    let mut hospital_repo = DatabaseHospitalRepository::new(pool.clone());
    let mut group_repo = DatabaseGroupRepository::new(pool.clone());
    let mut patient_repo = DatabasePatientRepository::new(pool.clone());

    let args: Vec<String> = env::args().collect();
    if args.iter().any(|arg| arg == "--setup") {
        group_repo.setup()
            .await
            .expect("Should be able to setup group repository");
        patient_repo.setup(|| async {
            // Have to do this gore so the patient table gets dropped, then the
            // hospital table gets dropped and setup, then the patient table
            // gets setup. Could be written better, but that's the way I did it.
                hospital_repo.setup()
                    .await
                    .expect("Should be able to setup hospital repository");
            })
            .await
            .expect("Should be able to setup patient repository");
    }

    // Actix web uses web::Data to share resources across requests, though they
    // must be wrapped in a Mutex for synchronization
    let hosp_service = web::Data::new(Mutex::new(HospitalService::new(hospital_repo)));
    let user_service = web::Data::new(Mutex::new(UserService::new(group_repo)));
    let patient_service = web::Data::new(Mutex::new(PatientService::new(
        patient_repo,
        ComplementService::new(RemoteComplementProvider::new("http://localhost:8081"))
    )));
    let oid = web::Data::new(openid_service); // non-writing service, so no mutex needed

    println!("Starting web server...");
    
    HttpServer::new(move || {
        App::new()
            .app_data(hosp_service.clone()) // app data is thread-safe
            .app_data(patient_service.clone())
            .app_data(oid.clone())
            .app_data(user_service.clone())
            // the session allows us to persist data across requests and associate
            // it with a single user. This demo uses a cookie to store all the
            // session data, as the Actix Session package does not support storing
            // in MSSQL. Ideally, we would store a key in the user's cookies, and
            // their associated data in MSSQL
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                Key::from("super-secret-key-that-must-be-at-least-64-bytes-long-so-I-guess-I-will-just-have-to-make-something-up".as_bytes())
            ))
            .configure(configure_jwt_routes)
            .configure(configure_openid_routes)
            .service(web::scope("/api/v1") // register API routes
                .wrap(HttpAuthentication::bearer(jwt_auth_middleware)) // apply JWT auth middleware
                .configure(configure_hospital_routes)
            )
        })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
