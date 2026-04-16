use rand::prelude::IndexedRandom;
use serde::{Deserialize, Serialize};
use crate::data::dates::GameDate;

/**

Represents A Full Place Someone Can Be Born

*/
#[derive(Serialize,Deserialize)]
pub struct Location {

    country: String,
    location: String,
    date: GameDate
}

#[derive(Serialize,Deserialize)]
pub struct Places{

    country: String,
    places: Vec<String>,

}



impl Location{

    pub fn new(country: &str,location: &str,date: GameDate)-> Location{
        Location {country: country.parse().unwrap(),location: location.parse().unwrap(), date}
    }


}

impl Places{
    pub fn new(country: &str) -> Places{

        Places{country:country.parse().unwrap(),places:Vec::new()}

    }

    pub fn add_place(&mut self,place: &str){

        self.places.push(place.to_string())

    }

    pub fn random_location(&self, date: GameDate) -> Location{

        let mut r = rand::rng();

        Location::new(&*self.country,self.places.choose(&mut r).unwrap(),date)
    }
}
