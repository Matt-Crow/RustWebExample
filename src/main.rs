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
    infrastructure::{controllers::{
        forecast_controller::configure_forecast_controller_routes, 
        anchor_controller::configure_anchor_controller_routes
    }, database::port_demo::set_up_tcp_listener_on},  
    infrastructure::database::connection::{
        create_client,
        create_config_from_env
    }
};

#[actix_web::main]
async fn main() -> std::io::Result<()> { // "()" is essentially "null"
    set_up_tcp_listener_on(1433); // this shows MSSQL is not listening on that port

    println!("Starting web server...");



    let config = match create_config_from_env() {
        Ok(c) => c,
        Err(error) => panic!("Error: {}", error)
    };
    println!("Config: {:#?}", config);

    match create_client(config).await {
        Ok(x) => println!("Yay: {:#?}", x),
        Err(x) => println!("Boo: {:#?}", x) // connection refused. TCP might be disabled
    }



    let sp = web::Data::new(ServiceProvider::default());

    HttpServer::new(move || {
        App::new()
            .app_data(sp.clone())
            .configure(configure_forecast_controller_routes)
            .service(web::scope("/api/v1")
                .configure(configure_forecast_controller_routes)
                .configure(configure_anchor_controller_routes)
            )
        })
        .bind(("127.0.0.1", 8080))? // "?" means "return error if this fails, else unwrap"
        .run()
        .await
}
