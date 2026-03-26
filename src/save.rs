use std::fs;
use std::io;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

const SAVE_DIR: &str = "flappy_rust";
const SAVE_FILE: &str = "highscore.json";

#[derive(Debug, Serialize, Deserialize, Default, Clone, Copy)]
pub struct SaveData {
    pub high_score: u32,
}

pub fn save_path() -> Option<PathBuf> {
    let base = dirs::data_local_dir()?.join(SAVE_DIR);
    Some(base.join(SAVE_FILE))
}

pub fn load() -> SaveData {
    let Some(path) = save_path() else {
        return SaveData::default();
    };
    match fs::read_to_string(&path) {
        Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
        Err(_) => SaveData::default(),
    }
}

pub fn save(data: &SaveData) -> io::Result<()> {
    let Some(path) = save_path() else {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "no data_local_dir",
        ));
    };
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(data).map_err(|e| {
        io::Error::new(io::ErrorKind::InvalidData, e.to_string())
    })?;
    fs::write(&path, json)
}
