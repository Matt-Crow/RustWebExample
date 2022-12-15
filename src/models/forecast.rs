use serde::Serialize;

#[derive(Serialize)]
pub struct Forecast {
    degrees_in_farenheight: f32,
    degrees_in_celsius: f32,
    location: String
}

impl Forecast {
    pub fn new(degrees_in_farenheight: f32, degrees_in_celsius: f32, location: &str) -> Forecast {
        Forecast {
            degrees_in_farenheight,
            degrees_in_celsius,
            location: location.to_string()
        }
    }

    /// gets this Forcast's degrees, measured in farenheight
    pub fn degrees_in_farenheight(&self) -> f32 {
        self.degrees_in_farenheight
    }
}