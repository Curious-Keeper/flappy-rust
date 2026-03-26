use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default, Clone, Copy)]
pub struct SaveData {
    pub high_score: u32,
}

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use std::fs;
    use std::io;
    use std::path::PathBuf;

    use super::SaveData;

    const SAVE_DIR: &str = "flappy_rust";
    const SAVE_FILE: &str = "highscore.json";

    fn save_path() -> Option<PathBuf> {
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
        let json = serde_json::to_string_pretty(data)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
        fs::write(&path, json)
    }
}

// Web: implemented in web/storage_plugin.js via miniquad_add_plugin (not wasm-bindgen).
#[cfg(target_arch = "wasm32")]
extern "C" {
    fn flappy_storage_load() -> i32;
    fn flappy_storage_save(score: i32);
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use std::io;

    use super::SaveData;

    pub fn load() -> SaveData {
        let v = unsafe { super::flappy_storage_load() };
        let high_score = if v < 0 { 0 } else { v as u32 };
        SaveData { high_score }
    }

    pub fn save(data: &SaveData) -> io::Result<()> {
        let score = i32::try_from(data.high_score).unwrap_or(i32::MAX);
        unsafe { super::flappy_storage_save(score) };
        Ok(())
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use native::{load, save};

#[cfg(target_arch = "wasm32")]
pub use wasm::{load, save};
