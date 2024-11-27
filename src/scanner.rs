use std::sync::mpsc::Sender;
use std::thread;
use std::{fs, path::PathBuf};
use std::fs::{self, DirEntry};
use std::path::Path;

use dirs_next as dirs;
use crate::logger; // 引入日志模块

pub fn scan_appdata(tx: Sender<(String, u64)>, folder_type: &str) {
    logger::log_info(&format!("开始扫描 {} 类型的文件夹", folder_type));
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
                            println!("Folder: {:?}, Size: {}", path.display(), size); // 打印文件夹路径和大小
                            tx.send((folder_name, size)).unwrap();
                        }
                    }
                }
            }
        });
    }
}

// 计算文件夹的总大小（递归）
fn calculate_folder_size(folder: &Path) -> u64 {
    let mut size = 0;

    // 遍历文件夹中的所有条目
    if let Ok(entries) = fs::read_dir(folder) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // 递归计算子文件夹的大小
                size += calculate_folder_size(&path);
            } else if path.is_file() {
                // 计算文件大小
                if let Ok(metadata) = entry.metadata() {
                    size += metadata.len();
                }
            }
        }
    }

    size
}

