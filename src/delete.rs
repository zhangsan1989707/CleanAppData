use std::fs;
use std::path::Path;

pub fn delete_folder(folder_path: &str) -> Result<(), String> {
    println!("Attempting to delete folder: {}", folder_path);

    let path = Path::new(folder_path);

    if !path.exists() {
        return Err("Folder does not exist.".to_string());
    }

    if path.is_dir() {
        fs::remove_dir_all(path).map_err(|e| e.to_string())
    } else {
        Err("The path is not a directory.".to_string())
    }
}
