use actix_web::{web, Responder};
use serde::{Serialize, Deserialize};
use crate::services::forecast_service::{self, get_forecast};

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

pub fn configure_forecast_controller_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1/forecast")
            .route("/{location}/{days}", web::get().to(forecast))
            .route("/to-celsius/{farenheight}", web::get().to(to_celsius))
            .route("/to-farenheight/{celsius}", web::get().to(to_farenheight))
    );
}

async fn forecast(days: web::Path<(String, u8)>) -> actix_web::Result<impl Responder> {
    let (location, num_days) = days.into_inner();
    Ok(web::Json(get_forecast(&location, num_days)))
}

async fn to_celsius(farenheight: web::Path<f32>) -> actix_web::Result<impl Responder> {
    let degrees_farenheight = farenheight.into_inner();
    let obj = Celsius {
        degrees_celsius: forecast_service::to_celsius(degrees_farenheight)
    };

    Ok(web::Json(obj))
}

async fn to_farenheight(celsius: web::Path<f32>) -> actix_web::Result<impl Responder> {
    let degrees_celsius = celsius.into_inner();
    let obj = Farenheight {
        degrees_farenheight: forecast_service::to_farenheight(degrees_celsius)
    };

    Ok(web::Json(obj))
}