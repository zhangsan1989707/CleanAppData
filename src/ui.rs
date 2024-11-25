use crate::delete;
use crate::scanner;
use crate::utils;
use eframe::egui::{self, Grid, ScrollArea};

pub struct AppDataCleaner {
    is_scanning: bool,
    current_folder: Option<String>,
    folder_data: Vec<(String, u64)>,
}

impl Default for AppDataCleaner {
    fn default() -> Self {
        Self {
            is_scanning: false,
            current_folder: None,
            folder_data: vec![],
        }
    }
}

// 中文字体
impl AppDataCleaner {
    fn setup_custom_fonts(&self, ctx: &egui::Context) {
        use eframe::egui::{FontData, FontDefinitions, FontFamily};

        let mut fonts = FontDefinitions::default();

        // 替换默认字体为自定义字体
        fonts.font_data.insert(
            "custom_font".to_owned(),
            FontData::from_static(include_bytes!("../assets/SourceHanSansCN-Regular.otf")),
        );

        // 覆盖所有字体设置
        fonts
            .families
            .insert(FontFamily::Proportional, vec!["custom_font".to_owned()]);
        fonts
            .families
            .insert(FontFamily::Monospace, vec!["custom_font".to_owned()]);

        ctx.set_fonts(fonts);
    }
}

impl eframe::App for AppDataCleaner {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 调用字体设置方法
        self.setup_custom_fonts(ctx);
        egui::CentralPanel::default().show(ctx, |ui| {
            //ui.label("中文测试：你好，世界！");
            if ui.button("立即扫描").clicked() && !self.is_scanning {
                self.is_scanning = true;
                self.folder_data.clear();

                let (tx, rx) = std::sync::mpsc::channel();
                std::thread::spawn(move || scanner::scan_appdata(tx));

                while let Ok((folder, size)) = rx.recv() {
                    self.folder_data.push((folder, size));
                }

                self.is_scanning = false;
                self.current_folder = None;
            }

            if self.is_scanning {
                ui.label("扫描中...");
            }

            ScrollArea::vertical().show(ui, |ui| {
                Grid::new("folders_table").striped(true).show(ui, |ui| {
                    ui.label("文件夹");
                    ui.label("大小");
                    ui.label("使用软件");
                    ui.label("操作");
                    ui.end_row();

                    for (folder, size) in &self.folder_data {
                        ui.label(folder);
                        ui.label(utils::format_size(*size));
                        ui.label("未知");

                        if ui.button("彻底删除").clicked() {
                            // 删除逻辑
                            if let Err(err) = delete::delete_folder(folder_name) {
                                eprintln!("Error: {}", err);
                            }
                        }
                        if ui.button("移动").clicked() {
                            // 移动逻辑
                        }
                        ui.end_row();
                    }
                });
            });
        });
    }
}
