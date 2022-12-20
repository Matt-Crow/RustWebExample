use actix_web::{web, Responder};
use serde::{Serialize, Deserialize};
use crate::services::forecast_service::{self, get_forecast};

// Enables JSON conversion
#[derive(Deserialize)]
#[derive(Serialize)]
struct Farenheight {
    degrees_farenheight: f32
}

#[derive(Deserialize)]
#[derive(Serialize)]
struct Celsius {
    degrees_celsius: f32
}

/// Sets up routing for the /api/v1/forecast endpoints.
/// Note the mutable borrow of the `ServiceConfig`.
pub fn configure_forecast_controller_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/forecast")
            .route("/to-celsius/{farenheight}", web::get().to(to_celsius_handler))
            .route("/to-farenheight/{celsius}", web::get().to(to_farenheight_handler))
            .route("/{location}/{days}", web::get().to(forecast_handler)) // must go after others
    );
}

/// since this matches the route /{location}/{days}, web::Path<(String, u8)>
/// will take the template variables in the path, attempt to convert them to the
/// proper type, then store them as a tuple of a String and unsigned byte.
async fn forecast_handler(days: web::Path<(String, u8)>) -> actix_web::Result<impl Responder> {
    let (location, num_days) = days.into_inner(); // deconstruct the tuple
    Ok(web::Json(get_forecast(&location, num_days))) // return a 200 response
                                                     // containing the forecast
                                                     // converted to JSON
}

async fn to_celsius_handler(farenheight: web::Path<f32>) -> actix_web::Result<impl Responder> {
    let degrees_farenheight = farenheight.into_inner();
    let obj = Celsius {
        degrees_celsius: forecast_service::to_celsius(degrees_farenheight)
    };

    Ok(web::Json(obj))
}

async fn to_farenheight_handler(celsius: web::Path<f32>) -> actix_web::Result<impl Responder> {
    let degrees_celsius = celsius.into_inner();
    let obj = Farenheight {
        degrees_farenheight: forecast_service::to_farenheight(degrees_celsius)
    };

    Ok(web::Json(obj))
}