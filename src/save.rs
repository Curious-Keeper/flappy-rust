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
        let json = serde_json::to_string_pretty(data)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
        fs::write(&path, json)
    }
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use std::io;

    use super::SaveData;

    const LS_KEY: &str = "flappy_rust_high_score";

    pub fn load() -> SaveData {
        let high_score = web_sys::window()
            .and_then(|w| w.local_storage().ok())
            .flatten()
            .and_then(|s| s.get_item(LS_KEY).ok().flatten())
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);
        SaveData { high_score }
    }

    pub fn save(data: &SaveData) -> io::Result<()> {
        let Some(win) = web_sys::window() else {
            return Err(io::Error::new(io::ErrorKind::Other, "no window"));
        };
        let Some(storage) = win
            .local_storage()
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "localStorage"))?
        else {
            return Err(io::Error::new(io::ErrorKind::Other, "no storage"));
        };
        storage
            .set_item(LS_KEY, &data.high_score.to_string())
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "set_item"))?;
        Ok(())
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use native::{load, save};

#[cfg(target_arch = "wasm32")]
pub use wasm::{load, save};
