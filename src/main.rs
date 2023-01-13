// Declare which modules (folders) should be compiled / loaded.
// These are searched recursively to load any of their declared modules as well.
pub mod core; // can declare modules as public in case other programs need them
mod infrastructure;

use actix_web::{
    HttpServer, 
    App,
    web
};
use actix_web_httpauth::middleware::HttpAuthentication;
use crate::{
    core::service_provider::ServiceProvider,
    infrastructure::{routes::configure_hospital_routes, authentication::jwt::{jwt_auth_middleware, configure_jwt_routes}, database::connection::{create_config_from_env, create_client}}
};

#[actix_web::main]
async fn main() -> std::io::Result<()> { // "()" is essentially "null"
    let mssql_config = create_config_from_env().expect("Failed to read MSSQL config!");
    let client = create_client(mssql_config).await;
    match client {
        Ok(client) => println!("MSSQL client: {:#?}", client),
        Err(error) => println!("Error creating MSSQL client: {:#?}", error)
    }

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
        .bind(("127.0.0.1", 8080))? // "?" means "return error if this fails, else unwrap"
        .run()
        .await
}


/*
trait Demo {

}

struct A {

}

impl Demo for A {
    
}

struct B {

}

impl Demo for B {

}

fn pick_demo<T>(use_a: bool) -> Box<T> 
where
    T: Demo
{
    if use_a {
        Box::new(A {})
    } else {
        Box::new(B {})
    }
}
*/