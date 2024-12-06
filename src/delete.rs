use std::fs;
use std::path::PathBuf;
use crate::logger;

/// 删除文件夹，接受 `PathBuf` 类型
pub fn delete_folder(folder_path: &PathBuf) -> Result<(), String> {
    let folder_path_str = folder_path.to_string_lossy();
    println!("尝试删除文件夹: {}", folder_path_str);
    logger::log_info(&format!("尝试删除文件夹: {}", folder_path_str));

    if !folder_path.exists() {
        let error_msg = format!("文件夹不存在: {}", folder_path_str);
        println!("{}", error_msg);
        logger::log_error(&error_msg);
        return Err(error_msg);
    }

    if folder_path.is_dir() {
        fs::remove_dir_all(folder_path).map_err(|e| {
            let error_msg = format!("删除失败: {} - 错误: {}", folder_path_str, e);
            println!("{}", error_msg);
            logger::log_error(&error_msg);
            error_msg
        })
    } else {
        let error_msg = format!("路径不是目录: {}", folder_path_str);
        println!("{}", error_msg);
        logger::log_error(&error_msg);
        Err(error_msg)
    }
}
