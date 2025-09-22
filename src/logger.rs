use simplelog::{Config, LevelFilter, SimpleLogger, WriteLogger};
use std::fs::File;
use std::sync::Once;

static INIT_LOGGER: Once = Once::new(); // 确保日志系统只初始化一次

/// 日志上下文，用于结构化输出
pub struct LogContext {
    /// 操作名称
    pub operation: &'static str,
    /// 目标类型
    pub target_type: Option<String>,
    /// 目标名称
    pub target_name: Option<String>,
}

impl LogContext {
    /// 创建新的日志上下文
    pub fn new(operation: &'static str) -> Self {
        Self {
            operation,
            target_type: None,
            target_name: None,
        }
    }

    /// 设置目标类型
    pub fn with_target_type(mut self, target_type: impl Into<String>) -> Self {
        self.target_type = Some(target_type.into());
        self
    }

    /// 设置目标名称
    pub fn with_target_name(mut self, target_name: impl Into<String>) -> Self {
        self.target_name = Some(target_name.into());
        self
    }

    /// 构造完整日志前缀
    pub fn prefix(&self) -> String {
        let mut prefix = format!("[{}]", self.operation);
        
        if let Some(target_type) = &self.target_type {
            prefix.push_str(&format!(" {}", target_type));
            
            if let Some(target_name) = &self.target_name {
                prefix.push_str(&format!(":{}", target_name));
            }
        }
        
        format!("{} -", prefix)
    }
}

pub fn init_logger(log_to_file: bool) {
    INIT_LOGGER.call_once(|| {
        if log_to_file {
            let _ = WriteLogger::init(
                LevelFilter::Info,
                Config::default(),
                File::create("cleanappdata.log").expect("无法创建日志文件"),
            );
        } else {
            let _ = SimpleLogger::init(LevelFilter::Info, Config::default());
        }
    });
}

pub fn log_info(message: &str) {
    log::info!("{}", message);
}

pub fn log_error(message: &str) {
    log::error!("{}", message);
}

pub fn log_structured_info(ctx: &LogContext, message: &str) {
    log::info!("{} {}", ctx.prefix(), message);
}

pub fn log_structured_error(ctx: &LogContext, message: &str) {
    log::error!("{} {}", ctx.prefix(), message);
}

pub fn log_structured_warn(ctx: &LogContext, message: &str) {
    log::warn!("{} {}", ctx.prefix(), message);
}

pub fn log_structured_debug(ctx: &LogContext, message: &str) {
    log::debug!("{} {}", ctx.prefix(), message);
}

/// 将API密钥进行部分掩码处理
pub fn mask_api_key(api_key: &str) -> String {
    if api_key.len() > 8 {
        format!("{}...{}", 
            &api_key[0..4], 
            &api_key[api_key.len() - 4..])
    } else {
        "***".to_string()
    }
}
