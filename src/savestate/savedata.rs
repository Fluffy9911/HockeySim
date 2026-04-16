use std::error::Error;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::data::player::NameData;
use crate::savestate::savedata;

// =========================
// CORE CONFIG
// =========================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CoreConfig {
    pub sim_id: String,
    pub data_id: String,
    pub version_id: String,
    pub game_id: String,
}

impl CoreConfig {
    pub fn new_def() -> CoreConfig {
        CoreConfig {
            sim_id: "HockeySim".to_string(),
            data_id: "SimData".to_string(),
            version_id: "0.1.0Alpha".to_string(),
            game_id: "HockeySim".to_string(),
        }
    }

    pub fn new(name: String) -> CoreConfig {
        CoreConfig {
            sim_id: name,
            data_id: "SimData".to_string(),
            version_id: "0.1.0Alpha".to_string(),
            game_id: "HockeySim".to_string(),
        }
    }

    pub fn load_name_data(&self) -> NameData {
        NameData::read_or_new(format!("{}_names", self.game_id).as_str())
    }
}

// =========================
// SAVE INFO (INDEX FILE)
// =========================

#[derive(Debug, Serialize, Deserialize)]
pub struct SaveInfo {
    pub saves: Vec<String>,
}

impl SaveInfo {
    pub fn new() -> SaveInfo {
        SaveInfo { saves: Vec::new() }
    }

    pub fn load() -> SaveInfo {
        let path = "data/saves.json";

        if Path::new(path).exists() {
            let data = fs::read_to_string(path)
                .expect("Failed to read saves.json");

            serde_json::from_str(&data)
                .unwrap_or_else(|_| SaveInfo::new())
        } else {
            let info = SaveInfo::new();

            fs::create_dir_all("data").unwrap();
            fs::write(path, serde_json::to_string_pretty(&info).unwrap())
                .expect("Failed to create saves.json");

            info
        }
    }

    pub fn create_save(&mut self, name: &str) -> CoreConfig {
        let saves_path = "data/saves";
        let index_path = "data/saves.json";

        fs::create_dir_all(saves_path).unwrap();

        if self.saves.contains(&name.to_string()) {
            panic!("Save already exists");
        }

        let save_dir = format!("{}/{}", saves_path, name);
        fs::create_dir_all(&save_dir).unwrap();

        let config = CoreConfig::new(name.to_string());

        fs::write(
            format!("{}/core.json", save_dir),
            serde_json::to_string_pretty(&config).unwrap(),
        )
            .unwrap();

        self.saves.push(name.to_string());

        fs::write(
            index_path,
            serde_json::to_string_pretty(self).unwrap(),
        )
            .unwrap();

        config
    }
}

// =========================
// FILE TYPES
// =========================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FileType {
    PLAYER_DATA,
    SAVE_DATA,
    TEAM_DATA,
    LEAGUE_DATA,
    CORE_DATA,
}

// =========================
// PATH HELPERS
// =========================

pub fn ensure_dir(path: &Path) {
    if let Err(e) = fs::create_dir_all(path) {
        panic!("couldn't create {}: {}", path.display(), e);
    }
}

pub fn create_path_for_type(core: &CoreConfig, t: &FileType) -> String {
    match t {
        FileType::CORE_DATA => format!("data/{}/{}", core.sim_id, core.game_id),
        FileType::LEAGUE_DATA => format!("data/{}/{}/League", core.sim_id, core.game_id),
        FileType::PLAYER_DATA => format!("data/{}/{}/Player", core.sim_id, core.game_id),
        FileType::SAVE_DATA => format!("data/{}/{}/Save", core.sim_id, core.game_id),
        FileType::TEAM_DATA => format!("data/{}/{}/Team", core.sim_id, core.game_id),
    }
}

pub fn file_path_from_type(core: &CoreConfig, t: &FileType, file: String) -> String {
    format!("{}/{}", create_path_for_type(core, t), file)
}

pub fn ensure_dir_type(core: &CoreConfig, t: &FileType) {
    let path = create_path_for_type(core, t);
    ensure_dir(Path::new(&path));
}

// =========================
// SAVE CONTEXT (ACTIVE SAVE)
// =========================

pub struct SaveContext {
    pub core: CoreConfig,
}

impl SaveContext {
    pub fn new(core: CoreConfig) -> Self {
        Self { core }
    }

    pub fn core(&self) -> &CoreConfig {
        &self.core
    }

    // ---------- FILE OPS ----------

    pub fn write_file(
        &self,
        file_type: FileType,
        file: &str,
        contents: &str,
    ) -> Result<(), Box<dyn Error>> {
        ensure_dir_type(&self.core, &file_type);
        let path = file_path_from_type(&self.core, &file_type, file.to_string());
        fs::write(path, contents)?;
        Ok(())
    }

    pub fn read_file(
        &self,
        file_type: FileType,
        file: &str,
    ) -> Result<String, Box<dyn Error>> {
        let path = file_path_from_type(&self.core, &file_type, file.to_string());
        Ok(fs::read_to_string(path)?)
    }

    pub fn write_struct<T>(
        &self,
        file_type: FileType,
        file: &str,
        data: &T,
    ) -> Result<(), Box<dyn Error>>
    where
        T: Serialize,
    {
        let json = serde_json::to_string_pretty(data)?;
        self.write_file(file_type, file, &json)
    }

    pub fn read_struct<T>(
        &self,
        file_type: FileType,
        file: &str,
    ) -> Result<T, Box<dyn Error>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let data = self.read_file(file_type, file)?;
        Ok(serde_json::from_str(&data)?)
    }

    pub fn exists(&self, file_type: FileType, file: &str) -> bool {
        let path = file_path_from_type(&self.core, &file_type, file.to_string());
        Path::new(&path).exists()
    }
}

// =========================
// COPY UTILITIES
// =========================

pub fn copy_file_between_saves(
    from: &CoreConfig,
    to: &CoreConfig,
    file_type: &FileType,
    file: &str,
) -> Result<(), Box<dyn Error>> {
    let src = file_path_from_type(from, file_type, file.to_string());
    let dst = file_path_from_type(to, file_type, file.to_string());

    if let Some(parent) = Path::new(&dst).parent() {
        fs::create_dir_all(parent)?;
    }

    fs::copy(src, dst)?;
    Ok(())
}

pub fn copy_type_between_saves(
    from: &CoreConfig,
    to: &CoreConfig,
    file_type: &FileType,
) -> Result<(), Box<dyn Error>> {
    let src_dir = create_path_for_type(from, file_type);
    let dst_dir = create_path_for_type(to, file_type);

    fs::create_dir_all(&dst_dir)?;

    for entry in fs::read_dir(src_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let file_name = path.file_name().unwrap();
            let dst_path = Path::new(&dst_dir).join(file_name);
            fs::copy(path, dst_path)?;
        }
    }

    Ok(())
}

pub fn clone_save(
    from: &CoreConfig,
    to: &CoreConfig,
) -> Result<(), Box<dyn Error>> {
    let types = [
        FileType::CORE_DATA,
        FileType::PLAYER_DATA,
        FileType::TEAM_DATA,
        FileType::LEAGUE_DATA,
        FileType::SAVE_DATA,
    ];

    for t in types.iter() {
        copy_type_between_saves(from, to, t)?;
    }

    Ok(())
}

// =========================
// INITIAL SETUP
// =========================

pub fn create_initial_state(core: &mut CoreConfig) -> NameData {
    savedata::ensure_dir_type(core, &FileType::CORE_DATA);
    savedata::ensure_dir_type(core, &FileType::TEAM_DATA);
    savedata::ensure_dir_type(core, &FileType::PLAYER_DATA);
    savedata::ensure_dir_type(core, &FileType::LEAGUE_DATA);
    savedata::ensure_dir_type(core, &FileType::SAVE_DATA);

    core.load_name_data()
}