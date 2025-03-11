use std::env;
use std::path::Path;
use std::sync::mpsc::Sender;
use std::thread;
use std::{fs, path::PathBuf};

use crate::logger;
use dirs_next as dirs; // 引入日志模块

pub fn scan_appdata(tx: Sender<(String, u64)>, folder_type: &str) {
    println!("开始扫描 {} 类型的文件夹", folder_type);
    // 记录日志
    logger::log_info(&format!("开始扫描 {} 类型的文件夹", folder_type));

    // 根据 folder_type 确定要扫描的目录
    let appdata_dir = match folder_type {
        "Roaming" => dirs::data_dir(),
        "Local" => dirs::cache_dir(),
        "LocalLow" => {
            // 通过 APPDATA 环境变量推导路径
            env::var("APPDATA").ok().and_then(|apdata| {
                let appdata_path = PathBuf::from(apdata);
                // 获取上级目录（即 AppData 文件夹）
                appdata_path
                    .parent()
                    .map(|appdata_dir| appdata_dir.join("LocalLow"))
            })
        }

        // 未知类型返回 None
        _ => None,
    };

    // 如果找到有效的目录，开始扫描
    if let Some(appdata_dir) = appdata_dir {
        thread::spawn(move || {
            if let Ok(entries) = fs::read_dir(&appdata_dir) {
                for entry in entries.flatten() {
                    if let Ok(metadata) = entry.metadata() {
                        if metadata.is_dir() {
                            let folder_name = entry.file_name().to_string_lossy().to_string();
                            let size = calculate_folder_size(&entry.path());
                            // 发送文件夹大小数据
                            tx.send((folder_name, size)).unwrap();
                        }
                    }
                }
            }
            // 发送一个特殊标志，表示扫描完成
            tx.send(("__SCAN_COMPLETE__".to_string(), 0)).unwrap();
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
