use serde::Serialize;

#[derive(Serialize)] // enables JSON serialization
pub struct Forecast {
    degrees_in_farenheight: f32,
    degrees_in_celsius: f32,
    location: String // structs should maintain ownership of their members, 
                     // so use String instead of &str
}

impl Forecast {
    pub fn new(degrees_in_farenheight: f32, degrees_in_celsius: f32, location: &str) -> Forecast {
        Forecast {
            degrees_in_farenheight,
            degrees_in_celsius,
            location: String::from(location) // function borrows reference to 
                                             // location, so make a copy and 
                                             // take ownership of that copy
        }
    }

    /// gets this Forcast's degrees, measured in farenheight
    pub fn degrees_in_farenheight(&self) -> f32 {
        self.degrees_in_farenheight // defines getter
    }
}