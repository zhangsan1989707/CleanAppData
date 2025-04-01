use crate::stats::Stats;
use crate::stats_logger::StatsLogger;
use crate::yaml_loader::{load_folder_descriptions, FolderDescriptions};
use crate::{confirmation, ignore, logger, move_module, open, scanner, utils};
use eframe::egui::{self, Grid, ScrollArea};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender}; // 引入 StatsLogger 模块

pub struct ClearTabState {
    // 基础字段
    pub is_scanning: bool,
    pub folder_data: Vec<(String, u64)>,
    pub selected_appdata_folder: String,
    pub tx: Option<Sender<(String, u64)>>,
    pub rx: Option<Receiver<(String, u64)>>,
    pub total_size: u64,

    // 界面状态字段
    pub confirm_delete: Option<(String, bool)>,
    pub status: Option<String>,

    // 排序相关字段
    pub sort_criterion: Option<String>, // 排序标准:"name"或"size"
    pub sort_order: Option<String>,     // 排序顺序:"asc"或"desc"

    // 文件夹描述相关
    pub folder_descriptions: Option<FolderDescriptions>,
    pub yaml_error_logged: bool,
    pub ignored_folders: HashSet<String>,

    // 移动模块
    pub move_module: move_module::MoveModule,

    // 生成描述的回调函数
    generate_description_callback: Option<Box<dyn Fn(&str) + Send>>,
    generate_all_descriptions_callback: Option<Box<dyn Fn(&Vec<(String, u64)>, &str) + Send>>,

    // 新增字段
    pub stats: Stats,
    pub stats_logger: StatsLogger, // 新增字段
}

impl Default for ClearTabState {
    fn default() -> Self {
        let (tx, rx) = std::sync::mpsc::channel();

        Self {
            // 基础字段初始化
            is_scanning: false,
            folder_data: vec![],
            selected_appdata_folder: "Roaming".to_string(),
            tx: Some(tx),
            rx: Some(rx),
            total_size: 0,

            // 界面状态初始化
            confirm_delete: None,
            status: Some("未扫描".to_string()),

            // 排序相关初始化
            sort_criterion: None,
            sort_order: None,

            // 文件夹描述相关初始化
            folder_descriptions: None,
            yaml_error_logged: false,
            ignored_folders: ignore::load_ignored_folders(),

            // 移动模块初始化
            move_module: Default::default(),

            // 回调函数初始化为 None
            generate_description_callback: None,
            generate_all_descriptions_callback: None,

            // 新增字段初始化
            stats: Stats::new(),
            stats_logger: StatsLogger::new(PathBuf::from("stats.log")), // 初始化 StatsLogger
        }
    }
}

// 其他代码保持不变

impl ClearTabState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_generate_description_callback<F>(&mut self, callback: F)
    where
        F: Fn(&str) + Send + 'static,
    {
        self.generate_description_callback = Some(Box::new(callback));
    }

    pub fn set_generate_all_descriptions_callback<F>(&mut self, callback: F)
    where
        F: Fn(&Vec<(String, u64)>, &str) + Send + 'static,
    {
        self.generate_all_descriptions_callback = Some(Box::new(callback));
    }

    // 抽取文件夹操作逻辑到单独的方法
    fn handle_folder_operations(&mut self, ui: &mut egui::Ui, folder: &str, size: u64) {
        // 显示文件夹名称和大小
        if self.ignored_folders.contains(folder) {
            ui.add_enabled(
                false,
                egui::Label::new(egui::RichText::new(folder).color(egui::Color32::GRAY)),
            );
        } else {
            ui.label(folder);
        }
        ui.label(utils::format_size(size));

        // 显示描述
        self.show_folder_description(ui, folder);

        // 显示操作按钮
        self.show_folder_actions(ui, folder);
    }

    fn show_folder_description(&self, ui: &mut egui::Ui, folder: &str) {
        let description = self
            .folder_descriptions
            .as_ref()
            .and_then(|desc| desc.get_description(folder, &self.selected_appdata_folder));

        match description {
            Some(desc) => ui.label(desc),
            None => ui.label("无描述"),
        };
    }

    fn show_folder_actions(&mut self, ui: &mut egui::Ui, folder: &str) {
        let is_ignored = self.ignored_folders.contains(folder);

        if !is_ignored {
            if ui.button("彻底删除").clicked() {
                self.confirm_delete = Some((folder.to_string(), false));
                self.status = None;
            }
            if ui.button("移动").clicked() {
                self.move_module.show_window = true;
                self.move_module.folder_name = folder.to_string();
            }
            if ui.button("忽略").clicked() {
                self.ignored_folders.insert(folder.to_string());
                ignore::save_ignored_folders(&self.ignored_folders);
                logger::log_info(&format!("文件夹 '{}' 已被忽略", folder));
            }
        } else {
            ui.add_enabled(false, |ui: &mut egui::Ui| {
                let response1 = ui.button("彻底删除");
                let response2 = ui.button("移动");
                let response3 = ui.button("忽略");
                response1 | response2 | response3
            });
        }

        if ui.button("打开").clicked() {
            if let Some(base_path) = utils::get_appdata_dir(&self.selected_appdata_folder) {
                let full_path = base_path.join(folder);
                if let Err(err) = open::open_folder(&full_path) {
                    logger::log_error(&format!("无法打开文件夹: {}", err));
                }
            }
        }

        if ui.button("生成描述").clicked() {
            self.generate_description(folder);
        }
    }

    fn generate_description(&mut self, folder: &str) {
        if let Some(callback) = &self.generate_description_callback {
            self.status = Some(format!("正在为 {} 生成描述...", folder));
            // 传递实际的文件夹名和当前选中的AppData文件夹
            callback(folder);
        }
    }

    pub fn show_sort_controls(&mut self, ui: &mut egui::Ui) {
        // 添加排序按钮
        ui.menu_button("排序", |ui| {
            if ui.button("名称正序").clicked() {
                self.sort_criterion = Some("name".to_string());
                self.sort_order = Some("asc".to_string());
            }
            if ui.button("大小正序").clicked() {
                self.sort_criterion = Some("size".to_string());
                self.sort_order = Some("asc".to_string());
            }
            if ui.button("名称倒序").clicked() {
                self.sort_criterion = Some("name".to_string());
                self.sort_order = Some("desc".to_string());
            }
            if ui.button("大小倒序").clicked() {
                self.sort_criterion = Some("size".to_string());
                self.sort_order = Some("desc".to_string());
            }
        });

        // 计算总大小
        self.total_size = self.folder_data.iter().map(|(_, size)| size).sum();

        // 显示总大小
        ui.label(format!("总大小: {}", utils::format_size(self.total_size)));

        // 显示总清理数和总大小
        ui.label(format!(
            "已清理文件夹数量: {}",
            self.stats.cleaned_folders_count
        ));
        ui.label(format!(
            "总清理大小: {}",
            utils::format_size(self.stats.total_cleaned_size)
        ));
    }

    pub fn show_folder_grid(&mut self, ui: &mut egui::Ui) {
        Grid::new("folders_table").striped(true).show(ui, |ui| {
            ui.label("文件夹");
            ui.label("大小");
            ui.label("描述");
            ui.label("操作");
            ui.end_row();

            // 先排序
            if let Some(criterion) = &self.sort_criterion {
                self.folder_data.sort_by(|a, b| {
                    if *criterion == "name" {
                        if self.sort_order == Some("asc".to_string()) {
                            a.0.cmp(&b.0)
                        } else {
                            b.0.cmp(&a.0)
                        }
                    } else {
                        if self.sort_order == Some("asc".to_string()) {
                            a.1.cmp(&b.1)
                        } else {
                            b.1.cmp(&a.1)
                        }
                    }
                });
            }

            // 创建一个临时向量来存储需要处理的数据
            let folder_data = self.folder_data.clone();

            // 使用临时数据进行遍历
            for (folder, size) in folder_data {
                self.handle_folder_operations(ui, &folder, size);
                ui.end_row();
            }
        });
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        // 初始化if未加载folder descriptions
        if self.folder_descriptions.is_none() {
            self.folder_descriptions =
                load_folder_descriptions("folders_description.yaml", &mut self.yaml_error_logged);
        }

        // 删除确认弹窗逻辑
        confirmation::handle_delete_confirmation(
            ui.ctx(), // 使用当前上下文，而不是创建新的
            &mut self.confirm_delete,
            &self.selected_appdata_folder,
            &mut self.status,
            &mut self.folder_data,
            &mut self.stats,    // 传递 stats 参数
            &self.stats_logger, // 传递 stats_logger 参数
        );

        // 扫描按钮和生成描述按钮放在一起
        ui.horizontal(|ui| {
            if ui.button("立即扫描").clicked() && !self.is_scanning {
                self.is_scanning = true;
                self.folder_data.clear();
                self.status = Some("扫描中...".to_string());

                let tx = self.tx.clone().unwrap();
                let folder_type = self.selected_appdata_folder.clone();

                scanner::scan_appdata(tx, &folder_type);
            }

            // 一键生成所有描述按钮
            if ui.button("一键生成所有描述").clicked() {
                if let Some(callback) = &self.generate_all_descriptions_callback {
                    self.status = Some("正在生成描述...".to_string());
                    callback(&self.folder_data, &self.selected_appdata_folder);
                }
            }
        });

        // 接收扫描结果
        if let Some(rx) = &self.rx {
            while let Ok((folder, size)) = rx.try_recv() {
                // 检查是否接收到扫描完成标志
                if folder == "__SCAN_COMPLETE__" {
                    self.is_scanning = false;
                    self.status = Some("扫描完成".to_string());
                } else {
                    self.folder_data.push((folder, size));
                }
            }
        }

        // 显示状态
        if let Some(status) = &self.status {
            ui.label(status);
        }

        // 排序控件
        self.show_sort_controls(ui);

        // 文件夹列表
        ScrollArea::vertical().show(ui, |ui| {
            self.show_folder_grid(ui);
        });
    }

    // 设置选中的AppData文件夹
    pub fn set_selected_appdata_folder(&mut self, folder: String) {
        self.selected_appdata_folder = folder;
        self.folder_data.clear();
        self.is_scanning = false;
        self.status = Some("未扫描".to_string());
    }

    // 更新文件夹描述
    pub fn update_folder_descriptions(&mut self) {
        self.folder_descriptions =
            load_folder_descriptions("folders_description.yaml", &mut self.yaml_error_logged);
    }
}
