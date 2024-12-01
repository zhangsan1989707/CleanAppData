use simplelog::{Config, LevelFilter, SimpleLogger, WriteLogger};
use std::fs::File;
use std::sync::Once;

static INIT_LOGGER: Once = Once::new(); // 确保日志系统只初始化一次

pub fn init_logger(log_to_file: bool) {
    INIT_LOGGER.call_once(|| {
        if log_to_file {
            let _ = WriteLogger::init(
                LevelFilter::Info,
                Config::default(),
                File::create("appdata_cleaner.log").expect("无法创建日志文件"),
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
