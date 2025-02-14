use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::{collections::HashMap, env, fs, path::Path, thread, time::Duration};
use tokio;
use anyhow::{Result, Context};

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    name: String,
    model: ModelConfig,
    retry: RetryConfig,
    output: OutputConfig,
    prompt: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ModelConfig {
    url: String,
    name: String,
    api_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct RetryConfig {
    attempts: u32,
    delay: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct OutputConfig {
    descriptions_yaml: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct YamlData {
    name: String,
    Local: HashMap<String, String>,
    LocalLow: HashMap<String, String>,
    Roaming: HashMap<String, String>,
}

const CONFIG: Config = Config {
    name: String::from("Xch13"),
    model: ModelConfig {
        url: String::from("https://xxx/v1/chat/completions"),
        name: String::from("xxx"),
        api_key: String::from("Bearer xxx"),
    },
    retry: RetryConfig {
        attempts: 3,
        delay: 20,
    },
    output: OutputConfig {
        descriptions_yaml: String::from("folders_description.yaml"),
    },
    prompt: String::from(r#"# 角色：  Windows系统专家

# 背景信息：  在Windows操作系统中，AppData文件夹通常包含应用程序数据，这些数据可能是设置、配置文件、缓存文件等。

# 工作流程/工作任务：  
- 识别并分析AppData下的[xxx]文件夹中的[xxx]子文件夹。
- 研究该子文件夹的具体用途。
- 提供简洁准确的描述。

# 输入提示：  请简述Windows系统中AppData下的[{}]文件夹中的[{}]子文件夹的用途。

# 输出示例：  
- 软件：Chrome
作用：存储Chrome浏览器的用户数据，如历史记录、书签、密码等。
功能：用户数据管理


- 软件：Dropbox
作用：存储Dropbox同步的文件和文件夹。
功能：文件同步与存储


- 软件：Microsoft Office
作用：存储Office应用程序的设置和个性化选项。
功能：应用程序配置和个性化设置


# 注意事项：  
- 描述应简洁明了，不超过50字。
- 确保描述的准确性，避免误导用户。"#),
};

struct YamlManager {
    filename: String,
    data: YamlData,
}

impl YamlManager {
    fn new(filename: &str, name: &str) -> Result<Self> {
        let required_data = YamlData {
            name: name.to_string(),
            Local: HashMap::new(),
            LocalLow: HashMap::new(),
            Roaming: HashMap::new(),
        };

        let data = if Path::new(filename).exists() {
            let content = fs::read_to_string(filename)
                .with_context(|| format!("Failed to read file: {}", filename))?;
            serde_yaml::from_str(&content).unwrap_or(required_data)
        } else {
            required_data
        };

        let manager = YamlManager {
            filename: filename.to_string(),
            data,
        };
        manager.save()?;
        Ok(manager)
    }

    fn add_data(&mut self, section: &str, foldername: &str, description: &str) -> Result<()> {
        if section.is_empty() || foldername.is_empty() || description.is_empty() {
            println!("Skipping empty data: section={}, folder={}", section, foldername);
            return Ok(());
        }

        let section_map = match section {
            "Local" => &mut self.data.Local,
            "LocalLow" => &mut self.data.LocalLow,
            "Roaming" => &mut self.data.Roaming,
            _ => {
                println!("Invalid section: {}", section);
                return Ok(());
            }
        };

        let desc = if description.trim().is_empty() {
            "Unknown folder purpose"
        } else {
            description
        };

        section_map.insert(foldername.to_string(), desc.to_string());
        self.save()?;
        Ok(())
    }

    fn save(&self) -> Result<()> {
        let yaml_str = serde_yaml::to_string(&self.data)?;
        fs::write(&self.filename, yaml_str)?;
        Ok(())
    }
}

struct AppDataAnalyzer {
    config: &'static Config,
    yaml_manager: YamlManager,
}

impl AppDataAnalyzer {
    fn new() -> Result<Self> {
        let yaml_manager = YamlManager::new(&CONFIG.output.descriptions_yaml, &CONFIG.name)?;
        Ok(AppDataAnalyzer {
            config: &CONFIG,
            yaml_manager,
        })
    }

    async fn analyze_appdata_folder(&self, dir_1: &str, dir_2: &str) -> Result<String> {
        let client = reqwest::Client::new();
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&self.config.model.api_key)?,
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let data = serde_json::json!({
            "messages": [
                {
                    "role": "system",
                    "content": self.config.prompt
                },
                {
                    "role": "user",
                    "content": format!("请简述Windows系统中AppData下的[{}]文件夹中的[{}]子文件夹的用途。", dir_1, dir_2)
                }
            ],
            "model": self.config.model.name
        });

        let mut attempts = self.config.retry.attempts;
        while attempts > 0 {
            match client
                .post(&self.config.model.url)
                .headers(headers.clone())
                .json(&data)
                .send()
                .await
            {
                Ok(response) => {
                    let json = response.json::<serde_json::Value>().await?;
                    if let Some(content) = json["choices"][0]["message"]["content"].as_str() {
                        return Ok(content.to_string());
                    }
                    return Ok("Unknown folder purpose".to_string());
                }
                Err(e) => {
                    attempts -= 1;
                    if attempts > 0 {
                        println!(
                            "Analysis error, remaining attempts: {}, error: {}",
                            attempts, e
                        );
                        thread::sleep(Duration::from_secs(self.config.retry.delay));
                    } else {
                        println!("Analysis finally failed: {}", e);
                        return Ok("Error occurred during analysis".to_string());
                    }
                }
            }
        }
        Ok("Error occurred during analysis".to_string())
    }

    fn get_appdata_paths() -> Result<HashMap<String, String>> {
        let appdata = env::var("APPDATA").context("Failed to get APPDATA environment variable")?;
        let base_path = Path::new(&appdata)
            .parent()
            .context("Failed to get parent directory of APPDATA")?
            .to_string_lossy()
            .into_owned();

        let mut paths = HashMap::new();
        paths.insert(
            "Local".to_string(),
            Path::new(&base_path).join("Local").to_string_lossy().into_owned(),
        );
        paths.insert(
            "LocalLow".to_string(),
            Path::new(&base_path)
                .join("LocalLow")
                .to_string_lossy()
                .into_owned(),
        );
        paths.insert("Roaming".to_string(), appdata);

        Ok(paths)
    }

    fn list_directories(base_path: &str) -> Vec<String> {
        match fs::read_dir(base_path) {
            Ok(entries) => entries
                .filter_map(|entry| {
                    entry.ok().and_then(|e| {
                        if e.path().is_dir() {
                            e.file_name().into_string().ok()
                        } else {
                            None
                        }
                    })
                })
                .collect(),
            Err(e) => {
                println!("Error accessing {}: {}", base_path, e);
                vec!["Cannot access or empty directory".to_string()]
            }
        }
    }

    async fn process_directories(&mut self) -> Result<()> {
        let appdata_paths = Self::get_appdata_paths()?;

        for (section, path) in appdata_paths {
            let folders = Self::list_directories(&path);
            for folder in folders {
                if folder == "Cannot access or empty directory" {
                    continue;
                }

                if !self.yaml_manager.data.Local.contains_key(&folder)
                    && !self.yaml_manager.data.LocalLow.contains_key(&folder)
                    && !self.yaml_manager.data.Roaming.contains_key(&folder)
                {
                    let res = self.analyze_appdata_folder(&section, &folder).await?;
                    self.yaml_manager.add_data(&section, &folder, &res)?;
                    println!("New saved {} {}:\n{}", section, folder, res);
                    thread::sleep(Duration::from_secs(1));
                } else {
                    let existing_description = match section.as_str() {
                        "Local" => self.yaml_manager.data.Local.get(&folder),
                        "LocalLow" => self.yaml_manager.data.LocalLow.get(&folder),
                        "Roaming" => self.yaml_manager.data.Roaming.get(&folder),
                        _ => None,
                    };
                    if let Some(desc) = existing_description {
                        println!("Already saved {} {}:\n{}", section, folder, desc);
                    }
                }
            }
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut analyzer = AppDataAnalyzer::new()?;
    analyzer.process_directories().await?;
    Ok(())
}