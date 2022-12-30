// Declare which modules (folders) should be compiled / loaded.
// These are searched recursively to load any of their declared modules as well.
pub mod core; // can declare modules as public in case other programs need them
mod infrastructure;

use actix_web::{
    Responder, // "this can be converted to an HTTP response"
    HttpServer, 
    App, 
    get, 
    web
};
use tokio::net::TcpListener;
use crate::{
    infrastructure::controllers::{
        forecast_controller::configure_forecast_controller_routes, 
        anchor_controller::configure_anchor_controller_routes
    }, 
    core::services::service_provider::ServiceProvider, 
    infrastructure::database::connection::{
        create_client,
        create_config_from_env
    }
};

#[get("/")] // trait-based routing
async fn index() -> impl Responder {
    "This is the main page of the website."
} // todo try some front-end libraries here

#[actix_web::main]
async fn main() -> std::io::Result<()> { // "()" is essentially "null"
    tokio::spawn(async {
        let choke = TcpListener::bind("127.0.0.1:1433").await;

        match choke {
            Ok(foo) => {
                println!("Result: {:#?}", foo);
                match foo.accept().await {
                    Ok(_) => println!("OOPS! Looks like this thread is listening on port 1433! Resetting connection..."),
                    Err(e) => panic!("{}", e)
                }
            },
            Err(e) => panic!("{}", e)
        }        
    });

    println!("Starting web server...");

    let config = match create_config_from_env() {
        Ok(c) => c,
        Err(error) => panic!("Error: {}", error)
    };

    println!("Config: {:#?}", config);
    
    let connection = create_client(config).await;

    match connection {
        Ok(x) => println!("Yay: {:#?}", x),
        Err(x) => println!("Boo: {:#?}", x) // connection refused. TCP might be disabled
    }

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
        .run()
        .await
}
