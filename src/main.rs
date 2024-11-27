mod about;
mod confirmation;
mod delete;
mod scanner;
mod ui;
mod utils;
mod logger;  // 引入 logger 模块

use ui::AppDataCleaner;

fn main() -> Result<(), eframe::Error> {
    // 初始化日志
    logger::init_logger(true); // true 表示日志记录到文件，false 表示只输出到控制台

    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "AppData Cleaner",
        options,
        Box::new(|_| {
            logger::log_info("应用程序启动");
            Ok(Box::new(AppDataCleaner::default()))
        }),
    )
}
