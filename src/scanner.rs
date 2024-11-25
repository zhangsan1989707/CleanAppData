use dirs_next as dirs;
use std::fs;

pub fn scan_appdata(tx: std::sync::mpsc::Sender<(String, u64)>) {
    if let Some(appdata_dir) = dirs::data_dir() {
        // 正确使用 std::fs::read_dir 获取文件夹内容
        let entries =
            fs::read_dir(appdata_dir).unwrap_or_else(|_| fs::read_dir("/dev/null").unwrap());

        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    // 获取文件夹的名称
                    let folder_name = path
                        .file_name()
                        .and_then(|os_str| os_str.to_str())
                        .unwrap_or("<未知文件夹>")
                        .to_owned();

                    let size = calculate_folder_size(&path); // 计算文件夹大小
                    tx.send((folder_name, size)).unwrap();
                }
            }
        }
    }
}

fn calculate_folder_size(folder: &std::path::Path) -> u64 {
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
