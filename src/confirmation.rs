use crate::delete;
use crate::logger;
use crate::stats::Stats;
use crate::stats_logger::StatsLogger; // 引入 StatsLogger 模块
use crate::utils;
use eframe::egui;

pub fn handle_delete_confirmation(
    ctx: &egui::Context,
    confirm_delete: &mut Option<(String, bool)>,
    selected_appdata_folder: &str,
    status: &mut Option<String>,
    folder_data: &mut Vec<(String, u64)>, // 新增参数
    stats: &mut Stats,                    // 新增参数
    stats_logger: &StatsLogger,           // 新增参数
) {
    if let Some((folder_name, _)) = confirm_delete.clone() {
        let message = format!("确定要彻底删除文件夹 {} 吗？", folder_name);
        logger::log_info(&message);
        if let Some(confirm) = show_confirmation(ctx, &message, status) {
            if confirm {
                if let Some(base_path) = utils::get_appdata_dir(selected_appdata_folder) {
                    let full_path = base_path.join(&folder_name);
                    match delete::delete_folder(&full_path, stats, stats_logger) {
                        // 传递 stats 和 stats_logger
                        Ok(_) => {
                            // 检查文件夹是否已成功删除
                            if !full_path.exists() {
                                *status = Some(format!("文件夹 {} 已成功删除", folder_name));
                                println!("文件夹 {} 已成功删除", folder_name);
                                // 从 folder_data 中移除对应项目
                                folder_data.retain(|(name, _)| name != &folder_name);
                            } else {
                                *status = Some(format!("文件夹 {} 删除失败", folder_name));
                            }
                        }
                        Err(err) => {
                            eprintln!("Error: {}", err);
                            logger::log_error(&format!("Error: {}", err));
                            *status =
                                Some(format!("删除文件夹 {} 时发生错误: {}", folder_name, err));
                        }
                    }
                } else {
                    eprintln!("无法获取 {} 文件夹路径", selected_appdata_folder);
                    logger::log_error(&format!("无法获取 {} 文件夹路径", selected_appdata_folder));
                    *status = Some(format!("无法获取 {} 文件夹路径", selected_appdata_folder));
                }
            } else {
                *confirm_delete = None; // 用户选择关闭或取消
            }
        }
    }
}
