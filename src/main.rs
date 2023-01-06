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
    infrastructure::{
        anchor_routes::configure_anchor_routes, 
        database::port_demo::set_up_tcp_listener_on, 
        http_client
    },  
    infrastructure::database::connection::{
        create_client,
        create_config_from_env
    }
};

#[actix_web::main]
async fn main() -> std::io::Result<()> { // "()" is essentially "null"
    


    // testing to see if this can make outbound connections
    match http_client::get("http://google.com").await {
        Ok(_) => println!("Successfully made a request to Google"),
        Err(err) => panic!("Failed to make a request to Google: {}", err) 
    };

    set_up_tcp_listener_on(1433); // this shows MSSQL is not listening on that port

    let config = match create_config_from_env() {
        Ok(c) => c,
        Err(error) => panic!("Error: {}", error)
    };
    println!("Config: {:#?}", config);

    match create_client(config).await {
        Ok(x) => println!("Yay: {:#?}", x),
        Err(x) => println!("Boo: {:#?}", x) // connection refused. TCP might be disabled
    }

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
