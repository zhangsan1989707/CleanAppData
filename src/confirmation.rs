use eframe::egui;
use crate::logger;
use crate::delete;
use crate::utils;

pub fn show_confirmation(ctx: &egui::Context, message: &str) -> Option<bool> {
    let mut result = None;

    egui::Window::new("确认操作")
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.label(message);

            ui.horizontal(|ui| {
                if ui.button("确认").clicked() {
                    result = Some(true);
                }
                if ui.button("取消").clicked() {
                    result = Some(false);
                    println!("用户取消操作");
                }
            });
        });

    result
}

// 新增函数，用于显示成功提示
pub fn show_success(ctx: &egui::Context, message: &str) {
    egui::Window::new("操作成功")
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.label(message);
            if ui.button("关闭").clicked() {
                ui.close_menu(); // 关闭窗口
            }
        });
}

pub fn handle_delete_confirmation(
    ctx: &egui::Context,
    confirm_delete: &mut Option<(String, bool)>,
    selected_appdata_folder: &str,
) {
    if let Some((folder_name, _)) = confirm_delete {
        let message = format!("确定要彻底删除文件夹 {} 吗？", folder_name);
        logger::log_info(&message);
        if let Some(confirm) = show_confirmation(ctx, &message) {
            if confirm {
                if let Some(base_path) = utils::get_appdata_dir(selected_appdata_folder) {
                    let full_path = base_path.join(&folder_name); // 传递引用
                    if let Err(err) = delete::delete_folder(&full_path) {
                        eprintln!("Error: {}", err);
                        logger::log_error(&format!("Error: {}", err));
                    } else {
                        // 检查文件夹是否已成功删除
                        if !full_path.exists() {
                            let success_message = format!("文件夹 {} 已成功删除", folder_name);
                            show_success(ctx, &success_message);
                        }
                    }
                } else {
                    eprintln!("无法获取 {} 文件夹路径", selected_appdata_folder);
                    logger::log_error(&format!(
                        "无法获取 {} 文件夹路径",
                        selected_appdata_folder
                    ));
                }
            }
            *confirm_delete = None; // 清除状态
        }
    }
}