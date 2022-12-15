

pub struct Forecast {
    degrees_in_farenheight: f32,
    degrees_in_celsius: f32,
    location: String
}

impl Forecast {
    fn new(degrees_in_farenheight: f32, degrees_in_celsius: f32, location: &str) -> Forecast {
        Forecast {
            degrees_in_farenheight,
            degrees_in_celsius,
            location: location.to_string()
        }
    }

    pub fn degrees_in_farenheight(&self) -> f32 {
        self.degrees_in_farenheight
    }
}