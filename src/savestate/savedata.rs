use std::error::Error;
use std::fmt::format;
use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use crate::data::player::NameData;
use crate::savestate::savedata;

#[derive(Debug,Serialize,Deserialize)]
pub struct CoreConfig{

    pub sim_id: String,

    pub data_id: String,

    pub version_id: String,

pub game_id: String

}

#[derive(Debug,Serialize,Deserialize)]
pub struct SaveInfo{

    saves: Vec<String>

}


impl SaveInfo{

    pub fn new()->SaveInfo{
        SaveInfo{saves : Vec::new()}
    }

    pub fn create_save(&mut self,name: &str){

        let config = CoreConfig::new(name.parse().unwrap());

        let path = "data/saves.json";

        if !Path::new(path).exists(){
            fs::create_dir_all(path).unwrap();
            self.create_save(name);
        }else{



            write_struct(&config, &FileType::CORE_DATA, "core.json", &config).expect("TODO: panic message");
fs::write(path,serde_json::to_string_pretty(self).unwrap());
        }




    }

}

#[derive(Debug,Serialize,Deserialize)]
pub enum FileType {
    PLAYER_DATA,SAVE_DATA,TEAM_DATA,LEAGUE_DATA,CORE_DATA


}

impl CoreConfig {

    pub fn new_def() -> CoreConfig {
        CoreConfig{sim_id: "HockeySim".parse().unwrap(), data_id: "SimData".parse().unwrap(),version_id: "0.1.0Alpha".parse().unwrap(), game_id: "HockeySim".parse().unwrap() }
    }
    pub fn new(name:String) -> CoreConfig {
        CoreConfig{sim_id: name, data_id: "SimData".parse().unwrap(),version_id: "0.1.0Alpha".parse().unwrap(), game_id: "HockeySim".parse().unwrap() }
    }

    pub fn load_name_data(&self) -> NameData{

        let data = NameData::read_or_new(format!("{}_names",self.game_id).as_str());


        data

    }






}

pub fn ensure_dir(path: &Path)  {

    let dir = fs::create_dir_all(path);

    match dir {

        Err(why) => panic!("couldn't create {}: {}", path.display(), why),

        Ok(_ok) => ()

    }


}

pub fn create_path_for_type(core_config: &CoreConfig,t:&FileType)-> String {

    match t {
        FileType::CORE_DATA => format!("data/{}/{}",core_config.sim_id,core_config.game_id),
        FileType::LEAGUE_DATA => format!("data/{}/{}/League",core_config.sim_id,core_config.game_id),
        FileType::PLAYER_DATA => format!("data/{}/{}/Player",core_config.sim_id,core_config.game_id),
        FileType::SAVE_DATA => format!("data/{}/{}/Save",core_config.sim_id,core_config.game_id),
        FileType::TEAM_DATA => format!("data/{}/{}/Team",core_config.sim_id,core_config.game_id),

    }

}

pub fn file_path_from_type(core_config: &CoreConfig,t:&FileType,file:String) -> String{
    let pth = create_path_for_type(core_config,t);

     format!("{}/{}",pth,file)
}

pub fn ensure_dir_type(config:&CoreConfig, file_type:&FileType)  {

    let path = create_path_for_type(config,file_type);

    ensure_dir(Path::new(&path));



}



pub fn write_file(config:&CoreConfig, file_type:&FileType,file_dat: (String,String)){

    ensure_dir_type(config,file_type);

let file = file_path_from_type(config,file_type,file_dat.0);

   let msg =  fs::write(&file, file_dat.1);

    match msg {
        Err(e) => panic!("couldn't write to {}: {}", file, e),
        Ok(_ok) => ()

    }


}
pub fn query_file(config: &CoreConfig, file_type: &FileType, file: &str) -> bool {
    let path = file_path_from_type(config, file_type, file.to_string());
    Path::new(&path).exists()
}
pub fn read_file(config: &CoreConfig, file_type: &FileType, file: &str) -> Result<String, Box<dyn Error>> {
    let path = file_path_from_type(config, file_type, file.to_string());
    let contents = fs::read_to_string(path)?;
    Ok(contents)
}
pub fn read_struct<T>(
    config: &CoreConfig,
    file_type: &FileType,
    file: &str,
) -> Result<T, Box<dyn Error>>
where
    T: for<'de> Deserialize<'de>,
{
    let data = read_file(config, file_type, file)?;
    let parsed = serde_json::from_str::<T>(&data)?;
    Ok(parsed)
}
pub fn write_struct<T>(
    config: &CoreConfig,
    file_type: &FileType,
    file: &str,
    data: &T,
) -> Result<(), Box<dyn Error>>
where
    T: Serialize,
{
    ensure_dir_type(config, file_type);

    let path = file_path_from_type(config, file_type, file.to_string());

    let json = serde_json::to_string_pretty(data)?;
    fs::write(path, json)?;

    Ok(())
}


pub fn write_structs<T>(
    config: &CoreConfig,
    file_type: &FileType,
    files: Vec<(String, T)>,
) -> Result<(), Box<dyn Error>>
where
    T: Serialize,
{
    ensure_dir_type(config, file_type);

    for (file, data) in files {
        let path = file_path_from_type(config, file_type, file);
        let json = serde_json::to_string_pretty(&data)?;
        fs::write(path, json)?;
    }

    Ok(())
}
pub fn create_initial_state(core_data: &mut CoreConfig) -> NameData{
    savedata::ensure_dir_type(&core_data, &FileType::CORE_DATA);

    savedata::ensure_dir_type(&core_data, &FileType::TEAM_DATA);

    savedata::ensure_dir_type(&core_data, &FileType::PLAYER_DATA);

    savedata::ensure_dir_type(&core_data, &FileType::LEAGUE_DATA);

    savedata::ensure_dir_type(&core_data, &FileType::SAVE_DATA);

    core_data.load_name_data()
}
