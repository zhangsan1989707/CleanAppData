use eframe::egui;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Sender, Receiver};
use std::thread;
use std::sync::mpsc;

pub struct MoveModule {
    pub show_window: bool,
    pub folder_name: String,
    pub selected_path: Option<PathBuf>,
    pub progress: f32, // 复制进度
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
                    
                    if let Some(path) = &self.selected_path {
                        ui.horizontal(|ui| {
                            ui.label("目标路径:");
                            ui.text_edit_singleline(&mut path.display().to_string());
                        });
                    } else {
                        if ui.button("选择目标路径").clicked() {
                            // TODO: 调用路径选择对话框，获取用户选择的目标路径
                            self.selected_path = Some(PathBuf::from("C:/YourSelectedPath"));
                        }
                    }

                    if let Some(message) = &self.status_message {
                        ui.label(message);
                    }

                    if ui.button("确定").clicked() {
                        if let Some(target_path) = &self.selected_path {
                            if let Err(err) = self.move_folder_with_progress(&self.folder_name, target_path) {
                                self.status_message = Some(format!("错误: {}", err));
                            }
                        }
                    }
                    
                    if ui.button("取消").clicked() {
                        self.show_window = false;
                    }
                });
        }
    }

    fn move_folder_with_progress(&mut self, source: &str, target: &Path) -> Result<(), String> {
        let source_path = Path::new(source);
        if !source_path.exists() {
            return Err(format!("源文件夹 {} 不存在", source));
        }

        let (tx, rx) = mpsc::channel();
        self.status_message = Some("正在移动文件夹...".to_string());

        let target_path = target.to_path_buf();
        let source_path = source_path.to_path_buf();

        thread::spawn(move || {
            if let Err(err) = fs::create_dir_all(&target_path) {
                tx.send(Err(format!("无法创建目标目录: {}", err))).unwrap();
                return;
            }

            if let Err(err) = copy_dir_with_progress(&source_path, &target_path, &tx) {
                tx.send(Err(format!("复制失败: {}", err))).unwrap();
                return;
            }

            // 校验哈希值
            // TODO: 哈希校验逻辑

            // 删除原文件夹
            if let Err(err) = fs::remove_dir_all(&source_path) {
                tx.send(Err(format!("删除源目录失败: {}", err))).unwrap();
                return;
            }

            // 创建符号链接
            let output = std::process::Command::new("cmd")
                .args(["/C", "mklink", "/D", source_path.to_str().unwrap(), target_path.to_str().unwrap()])
                .output();

            match output {
                Ok(output) if output.status.success() => {
                    tx.send(Ok(format!(
                        "为 {} <<===>> {} 创建的符号链接",
                        source_path.display(),
                        target_path.display()
                    )))
                    .unwrap();
                }
                Ok(output) => {
                    tx.send(Err(format!(
                        "创建符号链接失败: {}",
                        String::from_utf8_lossy(&output.stderr)
                    )))
                    .unwrap();
                }
                Err(err) => {
                    tx.send(Err(format!("符号链接命令执行失败: {}", err))).unwrap();
                }
            }
        });

        for msg in rx {
            match msg {
                Ok(status) => {
                    self.status_message = Some(status);
                    self.progress = 1.0; // 完成
                }
                Err(err) => {
                    self.status_message = Some(err);
                    break;
                }
            }
        }

        Ok(())
    }
}

fn copy_dir_with_progress(
    source: &Path,
    target: &Path,
    tx: &Sender<Result<(), String>>,
) -> Result<(), String> {
    let entries = fs::read_dir(source).map_err(|err| format!("无法读取目录: {}", err))?;
    let total_entries = entries.count() as f32;
    let mut copied_entries = 0;

    for entry in fs::read_dir(source).map_err(|err| format!("无法读取目录: {}", err))? {
        let entry = entry.map_err(|err| format!("无法读取条目: {}", err))?;
        let file_type = entry.file_type().map_err(|err| format!("无法获取文件类型: {}", err))?;

        let src_path = entry.path();
        let dest_path = target.join(entry.file_name());

        if file_type.is_dir() {
            fs::create_dir_all(&dest_path).map_err(|err| format!("无法创建目录: {}", err))?;
            copy_dir_with_progress(&src_path, &dest_path, tx)?;
        } else {
            fs::copy(&src_path, &dest_path).map_err(|err| format!("无法复制文件: {}", err))?;
        }

        copied_entries += 1.0;
        let progress = copied_entries / total_entries;
        tx.send(Ok(())).unwrap(); // 更新进度
    }

    Ok(())
}
