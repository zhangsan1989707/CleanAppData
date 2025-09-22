//! AI 配置和处理模块
//! 
//! 此模块使用子模块组织 AI 功能的不同部分，清晰分离关注点

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use std::sync::mpsc::Sender;
use std::error::Error;
use std::env; // 添加环境变量相关的导入
use crate::logger::{self, LogContext};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

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

    // 导入环境变量相关的包
    use std::env;
    
    // 在config模块中添加从环境变量加载配置的方法
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

        /// 从环境变量加载配置
        pub fn load_from_env() -> Option<Self> {
            // 检查是否设置了AI_PROVIDER为bailian
            if let Ok(provider) = env::var("AI_PROVIDER") {
                if provider == "bailian" {
                    let api_key = env::var("DASHSCOPE_API_KEY").ok();
                    let model = env::var("AI_MODEL").ok();
                    let api_url = env::var("BAILIAN_API_URL").ok();
                     
                    // 确保必需的环境变量存在
                    if let (Some(api_key), Some(model), Some(api_url)) = (api_key, model, api_url) {
                        let mut config = Self::default();
                        config.model.url = api_url;
                        config.model.api_key = api_key;
                        config.model.model = model;
                         
                        return Some(config);
                    }
                }
            }
             
            None
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

            // 创建日志上下文
            let ctx = LogContext::new("API")
                .with_target_type(format!("{}", dir_1))
                .with_target_name(dir_2.to_string());

            logger::log_structured_info(&ctx, "开始生成文件夹描述");

            loop {
                attempts += 1;
                match self.try_get_description(dir_1, dir_2, &ctx).await {
                    Ok(description) => {
                        logger::log_structured_info(&ctx, 
                            &format!("成功获取描述 (字符数: {})", description.len()));
                        return Ok(description);
                    },
                    Err(e) => {
                        if attempts >= max_attempts {
                            logger::log_structured_error(&ctx, 
                                &format!("达到最大重试次数 {}/{}", attempts, max_attempts));
                            return Err(format!("达到最大重试次数 {}: {}", max_attempts, e).into());
                        }
                        logger::log_structured_warn(&ctx, 
                            &format!("请求失败 (尝试 {}/{}): {}，将在 {}s 后重试", 
                                attempts, max_attempts, e, delay.as_secs()));
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
            ctx: &LogContext,
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

            let masked_api_key = logger::mask_api_key(&self.config.model.api_key);

            // 只记录一次简化的API请求信息
            logger::log_structured_debug(ctx, &format!("请求细节: URL={}, 模型={}, API密钥={}", 
                self.config.model.url, self.config.model.model, masked_api_key));

            let response = self.client
                .post(&self.config.model.url)
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", self.config.model.api_key))
                .json(&request)
                .send()
                .await?;

            let status = response.status();
            
            if !status.is_success() {
                let error_text = response.text().await?;
                logger::log_structured_error(ctx, 
                    &format!("响应失败: HTTP {} - {}", status, error_text));
                return Err(format!("API请求失败: {} - {}", status, error_text).into());
            }

            logger::log_structured_debug(ctx, &format!("响应成功: HTTP {}", status));

            let chat_response: ChatResponse = response.json().await?;
            if let Some(choice) = chat_response.choices.first() {
                Ok(choice.message.content.clone())
            } else {
                logger::log_structured_error(ctx, "返回内容为空");
                Err("API返回空响应".into())
            }
        }

        /// 测试 API 连接
        pub async fn test_connection(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
            let ctx = LogContext::new("测试API")
                .with_target_type(format!("模型: {}", self.config.model.model));

            logger::log_structured_info(&ctx, "发送连接测试请求");

            let request = ChatRequest {
                messages: vec![Message {
                    role: "user".to_string(),
                    content: "测试连接".to_string(),
                }],
                model: self.config.model.model.clone(),
            };

            let masked_api_key = logger::mask_api_key(&self.config.model.api_key);

            logger::log_structured_debug(&ctx, 
                &format!("请求参数: URL={}, API密钥={}", 
                    self.config.model.url, masked_api_key));

            match self.client
                .post(&self.config.model.url)
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", self.config.model.api_key))
                .json(&request)
                .send()
                .await 
            {
                Ok(response) => {
                    let status = response.status();
                    let status_code = status.as_u16();
                    
                    logger::log_structured_info(&ctx, &format!("收到响应: HTTP {}", status_code));
                    
                    // 根据状态码返回具体信息
                    let result = match status_code {
                        200 => format!("连接成功 (HTTP 200 OK)"),
                        400 => {
                            let error_text = response.text().await?;
                            logger::log_structured_error(&ctx, &format!("请求错误: {}", error_text));
                            format!("请求错误 (HTTP 400): {}", error_text)
                        },
                        401 => {
                            logger::log_structured_error(&ctx, "认证失败: 密钥无效");
                            format!("认证失败 (HTTP 401): API密钥无效或已过期")
                        },
                        403 => {
                            logger::log_structured_error(&ctx, "API权限错误: 拒绝访问");
                            format!("拒绝访问 (HTTP 403): 没有权限访问此资源")
                        },
                        404 => {
                            logger::log_structured_error(&ctx, "API地址错误: 资源不存在");
                            format!("资源不存在 (HTTP 404): API端点URL可能不正确")
                        },
                        500 => {
                            logger::log_structured_error(&ctx, "API服务器错误");
                            format!("服务器内部错误 (HTTP 500): 请联系API服务提供商")
                        },
                        503 => {
                            logger::log_structured_error(&ctx, "API服务暂时不可用");
                            format!("服务暂时不可用 (HTTP 503): 服务器可能过载或正在维护")
                        },
                        _ => {
                            logger::log_structured_error(&ctx, &format!("未知响应状态: {}", status_code));
                            format!("未知响应 (HTTP {}): 请检查API文档", status_code)
                        }
                    };
                    
                    Ok(result)
                },
                Err(e) => {
                    // 处理网络连接问题
                    let result = if e.is_timeout() {
                        logger::log_structured_error(&ctx, &format!("连接超时: {}", e));
                        format!("连接超时: 请检查网络连接或API服务是否可用")
                    } else if e.is_connect() {
                        logger::log_structured_error(&ctx, &format!("连接失败: {}", e));
                        format!("连接失败: 无法连接到API服务器，请检查URL是否正确")
                    } else {
                        logger::log_structured_error(&ctx, &format!("请求错误: {}", e));
                        format!("请求错误: {}", e)
                    };
                    
                    Ok(result)
                }
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

    /// 取消标志，用于中断长时间运行的操作
    cancel_flag: Arc<AtomicBool>,
}

impl AIHandler {
    /// 创建新的 AI 处理器
    pub fn new(config: AIConfig, tx: Option<Sender<(String, String, String)>>) -> Self {
        let ctx = LogContext::new("系统").with_target_type("AI处理器");
        logger::log_structured_info(&ctx, &format!("初始化处理器，使用模型: {}", config.model.model));
        
        Self {
            client: AIClient::new(config.clone()),
            config,
            tx,
            cancel_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    /// 设置取消标志，用于中断正在进行的处理
    pub fn cancel_processing(&self) {
        self.cancel_flag.store(true, Ordering::SeqCst);
        let ctx = LogContext::new("系统").with_target_type("AI处理器");
        logger::log_structured_info(&ctx, "收到取消请求，将在下一轮处理后停止");
    }

    /// 重置取消标志，用于后续操作
    pub fn reset_cancel_flag(&self) {
        self.cancel_flag.store(false, Ordering::SeqCst);
    }

    /// 检查是否应该取消当前操作
    fn should_cancel(&self) -> bool {
        self.cancel_flag.load(Ordering::SeqCst)
    }

    /// 检查文件夹是否已有描述
    fn has_existing_description(&self, folder_name: &str, selected_folder: &str) -> bool {
        match selected_folder {
            "Local" => self.config.Local.contains_key(folder_name),
            "LocalLow" => self.config.LocalLow.contains_key(folder_name),
            "Roaming" => self.config.Roaming.contains_key(folder_name),
            _ => false,
        }
    }

    /// 处理单个文件夹描述生成
    pub async fn generate_single_description(
        &mut self,
        folder_name: String,
        selected_folder: String,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let ctx = LogContext::new("描述生成")
            .with_target_type(selected_folder.clone())
            .with_target_name(folder_name.clone());

        logger::log_structured_info(&ctx, "开始处理");

        match self.client.get_folder_description(&selected_folder, &folder_name).await {
            Ok(description) => self.handle_success_response(&selected_folder, &folder_name, &description, &ctx),
            Err(e) => self.handle_error_response(&selected_folder, &folder_name, e, &ctx),
        }
    }

    /// 处理成功响应
    fn handle_success_response(
        &mut self,
        selected_folder: &str,
        folder_name: &str,
        description: &str,
        ctx: &LogContext,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        // 更新配置中的描述
        self.update_folder_description(selected_folder, folder_name, description);
        
        // 保存并通知UI
        if let Err(e) = self.save_config_and_notify(selected_folder, folder_name, description, ctx) {
            logger::log_structured_error(ctx, &format!("保存失败: {}", e));
        } else {
            logger::log_structured_info(ctx, "处理完成");
        }
        Ok(())
    }

    /// 处理错误响应
    fn handle_error_response(
        &self,
        _selected_folder: &str,
        _folder_name: &str,
        error: Box<dyn Error + Send + Sync>,
        ctx: &LogContext,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        logger::log_structured_error(ctx, &format!("生成失败: {}", error));
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
        let ctx = LogContext::new("批量生成")
            .with_target_type(selected_folder.clone());

        // 重置取消标志，确保新的批量操作从头开始
        self.reset_cancel_flag();
        
        logger::log_structured_info(&ctx, &format!("开始处理 {} 个文件夹", folder_data.len()));
            
        let mut success_count = 0;
        let mut failed_count = 0;
        let mut skipped_count = 0;
        
        for (i, (folder, _)) in folder_data.iter().enumerate() {
            // 检查是否应该取消操作
            if self.should_cancel() {
                logger::log_structured_info(&ctx, "操作被用户取消");
                break;
            }

            // 创建文件夹上下文
            let folder_ctx = LogContext::new("批量生成")
                .with_target_type(format!("{}/{}", i + 1, folder_data.len()))
                .with_target_name(folder.clone());
            
            // 检查是否已有描述，如果有则跳过
            if self.has_existing_description(folder, &selected_folder) {
                logger::log_structured_info(&folder_ctx, "跳过 (已存在描述)");
                skipped_count += 1;
                continue;
            }

            logger::log_structured_info(&folder_ctx, "处理中");
            
            match self.generate_single_description(folder.clone(), selected_folder.clone()).await {
                Ok(_) => success_count += 1,
                Err(_) => failed_count += 1,
            }
        }
        
        if self.should_cancel() {
            logger::log_structured_info(&ctx, 
                &format!("操作已取消 - 成功: {}, 失败: {}, 跳过: {}, 未处理: {}", 
                    success_count, failed_count, skipped_count, 
                    folder_data.len() - success_count - failed_count - skipped_count));
        } else {
            logger::log_structured_info(&ctx, 
                &format!("处理完成 - 成功: {}, 失败: {}, 跳过: {}", 
                    success_count, failed_count, skipped_count));
        }
        
        // 重置取消标志，以便于后续操作
        self.reset_cancel_flag();
        
        Ok(())
    }

    /// 保存配置并通知 UI
    fn save_config_and_notify(
        &self,
        selected_folder: &str,
        folder_name: &str,
        description: &str,
        ctx: &LogContext,
    ) -> Result<(), Box<dyn Error>> {
        if let Ok(config_path) = AIConfig::get_config_path() {
            match self.config.save_to_file(config_path.to_str().unwrap()) {
                Ok(_) => {
                    // 发送更新消息到 UI
                    if let Some(tx) = &self.tx {
                        logger::log_structured_debug(ctx, "通知UI更新描述");
                        
                        let _ = tx.send((
                            selected_folder.to_string(),
                            folder_name.to_string(),
                            description.to_string(),
                        ));
                    }
                    Ok(())
                }
                Err(e) => Err(e.into())
            }
        } else {
            Err("无法获取配置文件路径".into())
        }
    }

    /// 测试 API 连接
    pub async fn test_connection(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        self.client.test_connection().await
    }

    /// 更新配置
    pub fn update_config(&mut self, config: AIConfig) {
        let ctx = LogContext::new("配置").with_target_type("AI处理器");
        logger::log_structured_info(&ctx, &format!("更新配置，模型: {}", config.model.model));
        self.config = config;
        self.client = AIClient::new(self.config.clone());
    }
}