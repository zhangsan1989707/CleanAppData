use std::path::Path;
use std::process::Command;

pub fn open_folder(folder_path: &Path) -> Result<(), String> {
    if !folder_path.exists() {
        return Err(format!("路径 {} 不存在", folder_path.display()));
    }

    let status = if cfg!(target_os = "windows") {
        Command::new("explorer").arg(folder_path).status()
    } else if cfg!(target_os = "macos") {
        Command::new("open").arg(folder_path).status()
    } else if cfg!(target_os = "linux") {
        Command::new("xdg-open").arg(folder_path).status()
    } else {
        return Err("不支持的操作系统".to_string());
    };

    match status {
        Ok(s) if s.success() => Ok(()),
        Ok(s) => Err(format!("打开文件夹失败，状态码: {}", s)),
        Err(e) => Err(format!("执行打开命令失败: {}", e)),
    }
}
