use std::sync::mpsc::Sender;
use std::thread;
use std::{fs, path::PathBuf};

use dirs_next as dirs;

pub fn scan_appdata(tx: Sender<(String, u64)>, folder_type: &str) {
    let appdata_dir = match folder_type {
        "Roaming" => dirs::data_dir(),
        "Local" => dirs::cache_dir(),
        "LocalLow" => Some(PathBuf::from("C:/Users/Default/AppData/LocalLow")), // 手动设置路径
        _ => None,
    };

    if let Some(appdata_dir) = appdata_dir {
        thread::spawn(move || {
            if let Ok(entries) = fs::read_dir(&appdata_dir) {
                for entry in entries.flatten() {
                    if let Ok(metadata) = entry.metadata() {
                        if metadata.is_dir() {
                            let folder_name = entry.file_name().to_string_lossy().to_string();
                            let size = calculate_folder_size(&entry.path());
                            tx.send((folder_name, size)).unwrap();
                        }
                    }
                }
            }
        });
    }
}

fn calculate_folder_size(folder: &PathBuf) -> u64 {
    let mut size = 0;
    if let Ok(entries) = fs::read_dir(folder) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                size += metadata.len();
            }
        }
    }
    size
}
