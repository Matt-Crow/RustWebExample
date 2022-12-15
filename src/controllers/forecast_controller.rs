use actix_web::{web, Responder};
use serde::{Serialize, Deserialize};

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
            .route("/to-celsius/{farenheight}", web::get().to(to_celsius))
            .route("/to-farenheight/{celsius}", web::get().to(to_farenheight))
    );
}

async fn to_celsius(farenheight: web::Path<f32>) -> actix_web::Result<impl Responder> {
    let degrees_farenheight = farenheight.into_inner();
    let obj = Celsius {
        degrees_celsius: ((degrees_farenheight - 32.0) * 5.0 / 9.0)
    };

    Ok(web::Json(obj))
}

async fn to_farenheight(celsius: web::Path<f32>) -> actix_web::Result<impl Responder> {
    let degrees_celsius = celsius.into_inner();
    let obj = Farenheight {
        degrees_farenheight: (degrees_celsius * 9.0 / 5.0 + 32.0)
    };

    Ok(web::Json(obj))
}