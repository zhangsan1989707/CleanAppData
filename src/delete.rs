use std::fs;
use std::path::Path;

/// 删除指定文件夹及其所有内容
pub fn delete_folder(folder_path: &str) -> Result<(), String> {
    let path = Path::new(folder_path);
    if path.exists() {
        fs::remove_dir_all(path).map_err(|e| format!("Failed to delete folder: {}", e))
    } else {
        Err("Folder does not exist.".to_string())
    }
}
