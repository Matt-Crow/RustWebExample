use core::time;
use std::thread;

use crate::models::forecast::Forecast;

/// converts from farenheight to celsius
pub fn to_celsius(farenheight: f32) -> f32 {
    (farenheight - 32.0) * 5.0 / 9.0
}

/// converts from celsius to farenheight
pub fn to_farenheight(celsius: f32) -> f32 {
    celsius * 9.0 / 5.0 + 32.0
}

/// Simulates making a request to a 3rd party service, then returns the weather
/// forecast for the next `days` days at the given `location`. This will lag for
/// a total of `days` seconds to simulate the 3rd party delay.
pub fn get_forecast(location: &str, days: u8) -> Vec<Forecast> {
    let mut forecasts: Vec<Forecast> = Vec::new();
    let mut d: f32;

    for i in 0..days {
        thread::sleep(time::Duration::from_secs(1));
        
        d = (i as f32) * 20.0;
        forecasts.push(Forecast::new(
            to_farenheight(d),
            d,
            location
        ));
    }

    forecasts
}