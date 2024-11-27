use std::fs;
use std::path::Path;
use crate::logger;

pub fn delete_folder(folder_path: &str) -> Result<(), String> {
    println!("尝试删除文件夹: {}", folder_path);
    logger::log_info(&format!("尝试删除文件夹: {}", folder_path));

    let path = Path::new(folder_path);

    if !path.exists() {
        //println!("文件夹不存在.".to_string());
        let error_msg = "文件夹不存在.".to_string();
        logger::log_error(&error_msg);
        return Err(error_msg);
    }

    if path.is_dir() {
        fs::remove_dir_all(path).map_err(|e| {
            println!("删除失败: {}", e);
            let error_msg = format!("删除失败: {}", e);
            logger::log_error(&error_msg);
            error_msg
        })
    } else {
        //println!("路径不是目录.".to_string());
        let error_msg = "路径不是目录.".to_string();
        logger::log_error(&error_msg);
        Err(error_msg)
    }
}
