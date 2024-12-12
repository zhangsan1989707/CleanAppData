use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct FolderDescriptions {
    pub Roaming: HashMap<String, String>,
    pub Local: HashMap<String, String>,
    pub LocalLow: HashMap<String, String>,
}

impl FolderDescriptions {
    pub fn load_from_yaml(file_path: &str) -> Result<Self, String> {
        let path = Path::new(file_path);
        if !path.exists() {
            return Err("YAML 文件未找到".to_string());
        }

        let content = fs::read_to_string(path).map_err(|e| format!("读取 YAML 文件失败: {}", e))?;

        let descriptions: FolderDescriptions =
            serde_yaml::from_str(&content).map_err(|e| format!("解析 YAML 文件失败: {}", e))?;

        Ok(descriptions)
    }

    pub fn get_description(&self, folder_name: &str, folder_type: &str) -> Option<String> {
        match folder_type {
            "Roaming" => self.Roaming.get(folder_name).cloned(),
            "Local" => self.Local.get(folder_name).cloned(),
            "LocalLow" => self.LocalLow.get(folder_name).cloned(),
            _ => None,
        }
    }
}
