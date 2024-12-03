mod about;        // 关于界面
mod confirmation; // 确认删除模块
mod delete;  // 引入删除模块
mod scanner; // 引入扫盘模块
mod ui;      // 引入 ui 模块
mod utils;   // 文件夹大小计算模块
mod logger;  // 引入日志模块
mod ignore; // 引入忽略模块

use ui::AppDataCleaner;

fn main() -> Result<(), eframe::Error> {
    // 初始化日志
    logger::init_logger(true); // true 表示日志记录到文件，false 表示只输出到控制台

    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "AppData 清理器",
        options,
        Box::new(|_| {
            logger::log_info("应用程序启动");
            Ok(Box::new(AppDataCleaner::default()))
        }),
    )
}
