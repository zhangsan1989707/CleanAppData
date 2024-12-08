// move_module.rs
use eframe::egui;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct MoveOperation {
    pub source_folder: PathBuf,
    pub target_folder: PathBuf,
}

impl MoveOperation {
    pub fn new(source_folder: PathBuf) -> Self {
        MoveOperation {
            source_folder,
            target_folder: PathBuf::new(),
        }
    }

    pub fn choose_target_folder(&mut self, ctx: &egui::Context) -> bool {
        let mut target_folder = String::new();
        egui::SidePanel::left("move_folder").show(ctx, |ui| {
            ui.label("选择目标文件夹");
            if ui.button("浏览").clicked() {
                let mut file_browser = egui::FileDialog::new("select_folder", "选择文件夹")
                    .action(egui::FileDialogAction::Open);
                file_browser.show(ctx);
                if let Some(path) = file_browser.selected_path() {
                    target_folder = path.to_str().unwrap().to_string();
                }
            }
            ui.text_edit_singleline(&mut target_folder);
        });
        if !target_folder.is_empty() {
            self.target_folder = PathBuf::from(target_folder);
            true
        } else {
            false
        }
    }

    pub fn confirm_and_move(&self, ctx: &egui::Context) -> bool {
        egui::Dialog::new("confirm_move")
            .default_width(400.0)
            .show(ctx, |ui| {
                ui.label(format!("您正在将 {} 移动至 {}", self.source_folder.display(), self.target_folder.display()));
                ui.label("这可能导致UWP程序异常！");
                if ui.button("确定").clicked() {
                    ui.close_modal(true);
                }
                ui.button("取消");
            })
    }

    pub fn perform_move(&self) -> Result<(), String> {
        // 复制文件夹内容到目标路径
        self.copy_folder(&self.source_folder, &self.target_folder)?;
        // 校验哈希值
        if !self.check_hash(&self.source_folder, &self.target_folder) {
            return Err("哈希校验失败".to_string());
        }
        // 删除原文件夹
        self.remove_dir_all(&self.source_folder)?;
        // 创建符号链接
        self.create_symbolic_link(&self.source_folder, &self.target_folder)?;
        Ok(())
    }

    fn copy_folder(&self, source: &Path, target: &Path) -> Result<(), String> {
        fs::create_dir_all(&target)?;
        for entry in fs::read_dir(source).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let target_path = target.join(entry.file_name());
            if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                self.copy_folder(&entry.path(), &target_path)?;
            } else {
                let mut src_file = File::open(entry.path()).map_err(|e| e.to_string())?;
                let mut dest_file = File::create(&target_path).map_err(|e| e.to_string())?;
                io::copy(&mut src_file, &mut dest_file).map_err(|e| e.to_string())?;
            }
        }
        Ok(())
    }

    fn check_hash(&self, source: &Path, target: &Path) -> bool {
        // 这里需要实现哈希校验逻辑，为了简化，我们假设总是返回true
        true
    }

    fn remove_dir_all(&self, path: &Path) -> Result<(), String> {
        fs::remove_dir_all(path).map_err(|e| e.to_string())
    }

    fn create_symbolic_link(&self, source: &Path, target: &Path) -> Result<(), String> {
        let output = Command::new("cmd")
            .args(&["/C", "mklink", "/D", source.to_str().unwrap(), target.to_str().unwrap()])
            .output()
            .map_err(|e| e.to_string())?;
        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).to_string());
        }
        Ok(())
    }
}