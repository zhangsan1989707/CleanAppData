use eframe::egui;
use crate::logger;
use crate::delete;
use crate::utils;

pub fn show_confirmation(ctx: &egui::Context, message: &str, status: &Option<String>, confirm_delete: &mut Option<(String, bool)>) -> Option<bool> {
    let mut result = None;

    egui::Window::new("确认操作")
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.label(message);

            // 显示状态信息
            if let Some(status_message) = status {
                ui.label(status_message);
            }

            ui.horizontal(|ui| {
                if status.is_some() {
                    if ui.button("关闭").clicked() {
                        *confirm_delete = None;
                        println!("用户关闭窗口");
                    }
                } else {
                    if ui.button("确认").clicked() {
                        result = Some(true);
                    }
                    if ui.button("取消").clicked() {
                        result = Some(false);
                        println!("用户取消操作");
                    }
                }
            });
        });

    result
}

pub fn handle_delete_confirmation(
    ctx: &egui::Context,
    confirm_delete: &mut Option<(String, bool)>,
    selected_appdata_folder: &str,
    status: &mut Option<String>,
) {
    if let Some((folder_name, _)) = confirm_delete {
        let message = format!("确定要彻底删除文件夹 {} 吗？", folder_name);
        logger::log_info(&message);
        if let Some(confirm) = show_confirmation(ctx, &message, status, confirm_delete) {
            if confirm {
                if let Some(base_path) = utils::get_appdata_dir(selected_appdata_folder) {
                    let full_path = base_path.join(&folder_name);
                    match delete::delete_folder(&full_path) {
                        Ok(_) => {
                            // 检查文件夹是否已成功删除
                            if !full_path.exists() {
                                *status = Some(format!("文件夹 {} 已成功删除", folder_name));
                            } else {
                                *status = Some(format!("文件夹 {} 删除失败", folder_name));
                            }
                        }
                        Err(err) => {
                            eprintln!("Error: {}", err);
                            logger::log_error(&format!("Error: {}", err));
                            *status = Some(format!("删除文件夹 {} 时发生错误: {}", folder_name, err));
                        }
                    }
                } else {
                    eprintln!("无法获取 {} 文件夹路径", selected_appdata_folder);
                    logger::log_error(&format!(
                        "无法获取 {} 文件夹路径",
                        selected_appdata_folder
                    ));
                    *status = Some(format!("无法获取 {} 文件夹路径", selected_appdata_folder));
                }
            }
        }
    }
}