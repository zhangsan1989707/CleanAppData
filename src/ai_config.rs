//! AI 配置和处理模块
//! 
//! 此模块使用子模块组织 AI 功能的不同部分，清晰分离关注点

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use std::sync::mpsc::Sender;
use std::error::Error;
use crate::logger;

/// 配置相关的数据结构
pub mod config {
    use super::*;

    /// AI 配置的主结构
    #[derive(Debug, Serialize, Deserialize, Clone)]
    #[allow(non_snake_case)]
    pub struct AIConfig {
        /// 模型和 API 配置
        pub model: ModelConfig,
        
        /// 重试策略配置
        pub retry: RetryConfig,
        
        /// Local 文件夹描述映射
        pub Local: HashMap<String, String>,
        
        /// LocalLow 文件夹描述映射
        pub LocalLow: HashMap<String, String>,
        
        /// Roaming 文件夹描述映射
        pub Roaming: HashMap<String, String>,
    }

    /// 模型和 API 配置
    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub struct ModelConfig {
        /// API 端点 URL
        pub url: String,
        
        /// API 密钥
        pub api_key: String,
        
        /// 使用的模型名称
        pub model: String,
        
        /// 系统提示词
        pub prompt: String,
    }

    /// 重试策略配置
    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub struct RetryConfig {
        /// 最大重试次数
        pub attempts: u32,
        
        /// 重试间隔(秒)
        pub delay: u32,
    }

    impl Default for AIConfig {
        fn default() -> Self {
            Self {
                model: ModelConfig {
                    url: "https://open.bigmodel.cn/api/paas/v4/chat/completions".to_string(),
                    api_key: "your_api_key_here".to_string(),
                    model: "glm-4-flash".to_string(),
                    prompt: r#"# 角色：Windows AppData分析专家

您是一个专业的WindowsAppData文件夹分析专家。您需要分析用户提供的AppData文件夹信息并按照固定格式回答。

## 输入格式验证规则
当用户输入包含以下要素时视为有效：
1. 包含"AppData"关键词
2. 包含主目录[Local|LocalLow|Roaming]之一
3. 包含具体的应用程序文件夹名称

## 输出格式
```
- 软件名称：<应用程序名称>
- 数据类别：[配置|缓存|用户数据|日志]
- 应用用途：<简要描述（限50字）>
- 管理建议：[是|否]可安全删除
```

## 示例对话
用户输入：请分析Windows系统中AppData下Local文件夹中的Microsoft文件夹

系统输出：
- 软件名称：Microsoft Office
- 数据类别：配置
- 应用用途：存储Office应用程序的本地设置和临时文件
- 管理建议：是可安全删除

## 处理指令
1. 对任何符合输入格式的查询，直接使用输出格式回答
2. 保持输出格式的严格一致性
3. 不添加任何额外解释或评论
4. 确保应用用途描述在50字以内

## 注意
仅当输入完全不符合格式要求时，才返回："请按照正确的输入格式提供查询信息""#
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
        /// 创建新的默认配置
        pub fn new() -> Self {
            Self::default()
        }

        /// 获取配置文件路径
        pub fn get_config_path() -> Result<std::path::PathBuf, Box<dyn Error>> {
            Ok(std::path::PathBuf::from("folders_description.yaml"))
        }

        /// 从文件加载配置
        pub fn load_from_file(path: &str) -> Result<Self, Box<dyn Error>> {
            let content = std::fs::read_to_string(path)?;
            Ok(serde_yaml::from_str(&content)?)
        }

        /// 保存配置到文件
        pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn Error>> {
            std::fs::write(path, serde_yaml::to_string(self)?)?;
            logger::log_info(&format!("配置文件已保存到: {}", path));
            Ok(())
        }

        /// 验证配置有效性
        pub fn validate(&self) -> Result<(), String> {
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
}

/// API 通信相关的数据结构和实现
pub mod api {
    use super::*;
    use super::config::AIConfig;

    /// 对话消息结构
    #[derive(Debug, Serialize, Deserialize)]
    pub struct Message {
        /// 角色: system, user, assistant
        pub role: String,
        
        /// 消息内容
        pub content: String,
    }

    /// 聊天请求结构
    #[derive(Debug, Serialize)]
    pub struct ChatRequest {
        /// 消息列表
        pub messages: Vec<Message>,
        
        /// 请求的模型名称
        pub model: String,
    }

    /// 聊天响应结构
    #[derive(Debug, Deserialize)]
    pub struct ChatResponse {
        /// 响应的选择列表
        pub choices: Vec<Choice>,
    }

    /// 响应选择结构
    #[derive(Debug, Deserialize)]
    pub struct Choice {
        /// 响应消息
        pub message: Message,
    }

    /// AI API 客户端
    #[derive(Debug)]
    pub struct AIClient {
        /// 配置信息
        config: AIConfig,
        
        /// HTTP 客户端
        client: reqwest::Client,
    }

    impl AIClient {
        /// 创建新的 AI 客户端
        pub fn new(config: AIConfig) -> Self {
            Self {
                config,
                client: reqwest::Client::new(),
            }
        }

        /// 获取文件夹描述，包含重试逻辑
        pub async fn get_folder_description(
            &self,
            dir_1: &str,
            dir_2: &str,
        ) -> Result<String, Box<dyn Error + Send + Sync>> {
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

        /// 尝试单次请求获取描述
        async fn try_get_description(
            &self,
            dir_1: &str,
            dir_2: &str,
        ) -> Result<String, Box<dyn Error + Send + Sync>> {
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

        /// 测试 API 连接
        pub async fn test_connection(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
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
}

// 重新导出主要的类型，保持向后兼容
pub use config::AIConfig;
pub use api::AIClient;

/// AI 处理逻辑的高级封装
#[derive(Debug)]
pub struct AIHandler {
    /// 配置信息
    config: AIConfig,
    
    /// API 客户端
    client: AIClient,
    
    /// UI 通信通道
    tx: Option<Sender<(String, String, String)>>,
}

impl AIHandler {
    /// 创建新的 AI 处理器
    pub fn new(config: AIConfig, tx: Option<Sender<(String, String, String)>>) -> Self {
        Self {
            client: AIClient::new(config.clone()),
            config,
            tx,
        }
    }

    /// 处理单个文件夹描述生成
    pub async fn generate_single_description(
        &mut self,
        folder_name: String,
        selected_folder: String,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        logger::log_info(&format!("开始为 {} 生成描述", folder_name));

        match self.client.get_folder_description(&selected_folder, &folder_name).await {
            Ok(description) => self.handle_success_response(&selected_folder, &folder_name, &description),
            Err(e) => self.handle_error_response(&selected_folder, &folder_name, e),
        }
    }

    /// 处理成功响应
    fn handle_success_response(
        &mut self,
        selected_folder: &str,
        folder_name: &str,
        description: &str,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        logger::log_info(&format!(
            "成功生成描述 - {}/{}: {}", 
            selected_folder, 
            folder_name, 
            description
        ));
        
        self.update_folder_description(selected_folder, folder_name, description);
        
        if let Err(e) = self.save_config_and_notify(selected_folder, folder_name, description) {
            logger::log_error(&format!("保存配置失败: {}", e));
        }
        Ok(())
    }

    /// 处理错误响应
    fn handle_error_response(
        &self,
        selected_folder: &str,
        folder_name: &str,
        error: Box<dyn Error + Send + Sync>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        logger::log_error(&format!(
            "生成描述失败 {}/{}: {}", 
            selected_folder,
            folder_name, 
            error
        ));
        Err(error)
    }

    /// 更新文件夹描述
    fn update_folder_description(&mut self, selected_folder: &str, folder_name: &str, description: &str) {
        match selected_folder {
            "Local" => { self.config.Local.insert(folder_name.to_string(), description.to_string()); }
            "LocalLow" => { self.config.LocalLow.insert(folder_name.to_string(), description.to_string()); }
            "Roaming" => { self.config.Roaming.insert(folder_name.to_string(), description.to_string()); }
            _ => {}
        };
    }

    /// 批量处理多个文件夹
    pub async fn generate_all_descriptions(
        &mut self,
        folder_data: Vec<(String, u64)>,
        selected_folder: String,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        for (folder, _) in folder_data {
            if let Err(e) = self.generate_single_description(folder.clone(), selected_folder.clone()).await {
                logger::log_error(&format!("处理文件夹 {} 时发生错误: {}", folder, e));
                continue;
            }
        }
        Ok(())
    }

    /// 保存配置并通知 UI
    fn save_config_and_notify(
        &self,
        selected_folder: &str,
        folder_name: &str,
        description: &str,
    ) -> Result<(), Box<dyn Error>> {
        if let Ok(config_path) = AIConfig::get_config_path() {
            match self.config.save_to_file(config_path.to_str().unwrap()) {
                Ok(_) => {
                    logger::log_info("配置文件保存成功");
                    // 发送更新消息到 UI
                    if let Some(tx) = &self.tx {
                        let _ = tx.send((
                            selected_folder.to_string(),
                            folder_name.to_string(),
                            description.to_string(),
                        ));
                    }
                    Ok(())
                }
                Err(e) => {
                    logger::log_error(&format!("保存配置失败: {}", e));
                    Err(e.into())
                }
            }
        } else {
            Err("无法获取配置文件路径".into())
        }
    }

    /// 测试 API 连接
    pub async fn test_connection(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.client.test_connection().await
    }

    /// 更新配置
    pub fn update_config(&mut self, config: AIConfig) {
        self.config = config;
        self.client = AIClient::new(self.config.clone());
    }
}