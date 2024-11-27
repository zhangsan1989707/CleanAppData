use simplelog::{Config, LevelFilter, SimpleLogger, WriteLogger};
use std::fs::File;

pub fn init_logger(log_to_file: bool) {
    if log_to_file {
        let _ = WriteLogger::init(
            LevelFilter::Info,
            Config::default(),
            File::create("appdata_cleaner.log").expect("无法创建日志文件"),
        );
    } else {
        let _ = SimpleLogger::init(LevelFilter::Info, Config::default());
    }
}

pub fn log_info(message: &str) {
    log::info!("{}", message);
}

pub fn log_error(message: &str) {
    log::error!("{}", message);
}
