use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use crate::logger;

// 为 AIConfig 添加 Clone trait
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AIConfig {
    pub name: String,
    pub model: ModelConfig,
    pub retry: RetryConfig,
    pub Local: HashMap<String, String>,
    pub LocalLow: HashMap<String, String>,
    pub Roaming: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelConfig {
    pub url: String,
    pub api_key: String,
    pub model: String,
    pub prompt: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RetryConfig {
    pub attempts: u32,
    pub delay: u32,
}

impl Default for AIConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            model: ModelConfig {
                url: "https://api.openai.com/v1".to_string(),
                api_key: "your_api_key_here".to_string(),
                model: "gpt-3.5-turbo".to_string(),
                prompt: r#"# 角色：  Windows系统专家

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
- 确保描述的准确性，避免误导用户。"#
                    .to_string(),
            },
            retry: RetryConfig {
                attempts: 3,
                delay: 20,
            },
            Local: HashMap::new(),
            LocalLow: HashMap::new(),
            Roaming: HashMap::new(),
        }
    }
}

impl AIConfig {
    pub fn new() -> Self {
        Self::default()
    }

    // 简化路径处理，直接使用固定路径
    pub fn get_config_path() -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
        Ok(std::path::PathBuf::from("folders_description.yaml"))
    }

    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: AIConfig = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        // 序列化配置
        let content = serde_yaml::to_string(self)?;

        // 写入文件
        std::fs::write(path, content)?;
        
        logger::log_info(&format!("配置文件已保存到: {}", path));
        
        Ok(())
    }

    pub fn create_default_config(
        name: Option<String>,
        api_key: Option<String>,
        model: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut config = Self::default();

        // 使用提供的参数更新配置
        if let Some(name) = name {
            config.name = name;
        }
        if let Some(api_key) = api_key {
            config.model.api_key = api_key;
        }
        if let Some(model) = model {
            config.model.model = model;
        }

        // 获取正确的配置文件路径
        let config_path = Self::get_config_path()?;
        config.save_to_file(config_path.to_str().unwrap())?;

        Ok(())
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("配置名称不能为空".to_string());
        }
        if self.model.url.trim().is_empty() {
            return Err("API地址不能为空".to_string());
        }
        if self.model.api_key.trim().is_empty() {
            return Err("API密钥不能为空".to_string());
        }
        if self.model.model.trim().is_empty() {
            return Err("模型名称不能为空".to_string());
        }
        Ok(())
    }
}

// API 请求相关结构体
#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct ChatRequest {
    pub messages: Vec<Message>,
    pub model: String,
}

#[derive(Debug, Deserialize)]
pub struct ChatResponse {
    pub choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
pub struct Choice {
    pub message: Message,
}

// AI 客户端结构体
#[derive(Debug)]
pub struct AIClient {
    config: AIConfig,
    client: reqwest::Client,
}

// 实现 AIClient
impl AIClient {
    pub fn new(config: AIConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }

    // 发送请求到 AI API 并处理重试
    pub async fn get_folder_description(
        &self,
        dir_1: &str,
        dir_2: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut attempts = 0;
        let max_attempts = self.config.retry.attempts;
        let delay = Duration::from_secs(self.config.retry.delay as u64);

        loop {
            attempts += 1;
            match self.try_get_description(dir_1, dir_2).await {
                Ok(description) => return Ok(description),
                Err(e) => {
                    if attempts >= max_attempts {
                        return Err(format!("达到最大重试次数 {}: {}", max_attempts, e).into());
                    }
                    crate::logger::log_error(&format!(
                        "API请求失败 (尝试 {}/{}): {}，将在 {}s 后重试",
                        attempts, max_attempts, e, delay.as_secs()
                    ));
                    tokio::time::sleep(delay).await;
                    continue;
                }
            }
        }
    }

    // 单次请求实现
    async fn try_get_description(
        &self,
        dir_1: &str,
        dir_2: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let request = ChatRequest {
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: self.config.model.prompt.clone(),
                },
                Message {
                    role: "user".to_string(),
                    content: format!(
                        "请简述Windows系统中AppData下的[{}]文件夹中的[{}]子文件夹的用途。",
                        dir_1, dir_2
                    ),
                },
            ],
            model: self.config.model.model.clone(),
        };

        let response = self.client
            .post(&self.config.model.url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.config.model.api_key))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!(
                "API请求失败: {} - {}", 
                response.status(),
                response.text().await?
            ).into());
        }

        let chat_response: ChatResponse = response.json().await?;
        if let Some(choice) = chat_response.choices.first() {
            Ok(choice.message.content.clone())
        } else {
            Err("API返回空响应".into())
        }
    }

    // 测试连接
    pub async fn test_connection(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let request = ChatRequest {
            messages: vec![Message {
                role: "user".to_string(),
                content: "测试连接".to_string(),
            }],
            model: self.config.model.model.clone(),
        };

        let response = self.client
            .post(&self.config.model.url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.config.model.api_key))
            .json(&request)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(format!("API连接测试失败: {}", response.status()).into())
        }
    }
}