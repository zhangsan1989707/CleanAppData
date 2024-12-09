use crate::logger;
use eframe::egui;
use native_dialog::FileDialog;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

pub struct MoveModule {
    pub show_window: bool,
    pub folder_name: String,            // 源文件夹路径
    pub selected_path: Option<PathBuf>, // 目标路径
    pub progress: f32,                  // 复制进度
    pub status_message: Option<String>, // 操作状态
}

impl Default for MoveModule {
    fn default() -> Self {
        Self {
            show_window: false,
            folder_name: String::new(),
            selected_path: None,
            progress: 0.0,
            status_message: None,
        }
    }
}

impl MoveModule {
    pub fn show_move_window(&mut self, ctx: &egui::Context) {
        if self.show_window {
            egui::Window::new("移动文件夹")
                .resizable(false)
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.label(format!("需要移动的文件夹: {}", self.folder_name));

                    // 显示目标路径选择
                    ui.horizontal(|ui| {
                        ui.label("目标路径:");
                        if let Some(path) = &self.selected_path {
                            ui.label(path.display().to_string());
                        }
                        if ui.button("选择目标路径").clicked() {
                            // 使用文件对话框选择目标路径
                            if let Ok(Some(path)) = FileDialog::new().show_open_single_dir() {
                                self.selected_path = Some(path);
                                println!(
                                    "目标路径选择: {}",
                                    self.selected_path.as_ref().unwrap().display()
                                );
                            }
                        }
                    });

                    // 显示状态信息
                    if let Some(message) = &self.status_message {
                        ui.label(message);
                    }

                    // 显示进度条
                    ui.add(egui::ProgressBar::new(self.progress).show_percentage());

                    // 操作按钮
                    if ui.button("确定").clicked() {
                        if let Some(target_path) = &self.selected_path {
                            self.start_move_folder(target_path.clone());
                        } else {
                            self.status_message = Some("请选择目标路径".to_string());
                        }
                    }

                    if ui.button("取消").clicked() {
                        self.show_window = false;
                    }
                });
        }
    }

    fn start_move_folder(&mut self, target_path: PathBuf) {
        // 确保源文件夹路径是完整路径
        let source_path = PathBuf::from(&self.folder_name);

        if !source_path.exists() {
            self.status_message = Some(format!("源文件夹不存在: {}", source_path.display()));
            println!("源文件夹不存在: {}", source_path.display());
            logger::log_error(&format!("源文件夹不存在: {}", source_path.display()));
            return;
        }

        let (tx, rx): (
            Sender<Result<String, String>>,
            Receiver<Result<String, String>>,
        ) = mpsc::channel();
        self.progress = 0.0;
        self.status_message = Some("正在移动文件夹...".to_string());

        // 启动后台线程执行移动逻辑
        thread::spawn(move || {
            println!(
                "开始复制: 从 {} 到 {}",
                source_path.display(),
                target_path.display()
            );

            if let Err(err) = fs::create_dir_all(&target_path) {
                let _ = tx.send(Err(format!("无法创建目标目录: {}", err)));
                return;
            }

            if let Err(err) = copy_dir_with_progress(&source_path, &target_path, &tx) {
                let _ = tx.send(Err(format!("复制失败: {}", err)));
                return;
            }

            // 删除原文件夹
            if let Err(err) = fs::remove_dir_all(&source_path) {
                let _ = tx.send(Err(format!("删除源目录失败: {}", err)));
                return;
            }

            // 创建符号链接
            let output = std::process::Command::new("cmd")
                .args([
                    "/C",
                    "mklink",
                    "/D",
                    source_path.to_str().unwrap(),
                    target_path.to_str().unwrap(),
                ])
                .output();

            match output {
                Ok(output) if output.status.success() => {
                    let _ = tx.send(Ok(format!(
                        "创建符号链接成功: {} -> {}",
                        source_path.display(),
                        target_path.display()
                    )));
                }
                Ok(output) => {
                    let _ = tx.send(Err(format!(
                        "创建符号链接失败: {}",
                        String::from_utf8_lossy(&output.stderr)
                    )));
                }
                Err(err) => {
                    let _ = tx.send(Err(format!("符号链接命令执行失败: {}", err)));
                }
            }
        });

        // 主线程接收消息并更新状态
        for msg in rx {
            match msg {
                Ok(status) => {
                    self.status_message = Some(status);
                    self.progress = 1.0;
                    break;
                }
                Err(err) => {
                    self.status_message = Some(err);
                    break;
                }
            }
        }
    }
}

// 带进度回调的目录复制函数
fn copy_dir_with_progress(
    source: &Path,
    target: &Path,
    tx: &Sender<Result<String, String>>,
) -> Result<(), String> {
    let entries: Vec<_> = fs::read_dir(source)
        .map_err(|err| format!("无法读取目录: {}", err))?
        .collect();

    let total_entries = entries.len() as f32;
    let mut copied_entries = 0.0;

    for entry in entries {
        let entry = entry.map_err(|err| format!("无法读取条目: {}", err))?;
        let file_type = entry
            .file_type()
            .map_err(|err| format!("无法获取文件类型: {}", err))?;

        let src_path = entry.path();
        let dest_path = target.join(entry.file_name());

        println!(
            "复制文件: 从 {} 到 {}",
            src_path.display(),
            dest_path.display()
        );

        if file_type.is_dir() {
            fs::create_dir_all(&dest_path).map_err(|err| format!("无法创建目录: {}", err))?;
            copy_dir_with_progress(&src_path, &dest_path, tx)?;
        } else {
            fs::copy(&src_path, &dest_path).map_err(|err| format!("无法复制文件: {}", err))?;
        }

        copied_entries += 1.0;
        let progress = copied_entries / total_entries;
        let _ = tx.send(Ok(format!("复制进度: {:.2}%", progress * 100.0)));
    }

    Ok(())
}
