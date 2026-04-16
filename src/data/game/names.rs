use std::string::ToString;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Division{
    name:String,
teams:i32


}

impl Division {
    pub fn atlantic() -> Division {
        Division{name: "Atlantic".parse().unwrap(), teams:8}
    }
}

#[derive(Serialize, Deserialize)]
pub struct Conference{

    name:String


}

impl Conference{

    pub fn east()-> Conference{
        Conference{name: "East".parse().unwrap() }
    }
    pub fn west()-> Conference{
        Conference{name: "West".parse().unwrap() }
    }


}

impl Division{

    pub fn pacific()-> Division{


        Division{name: "Pacific".parse().unwrap(), teams:8}
    }


}