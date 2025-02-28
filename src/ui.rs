use crate::{about, confirmation, ignore, logger, move_module, open, scanner, utils};
use crate::ai_config::{AIConfig, AIHandler};
use crate::yaml_loader::{load_folder_descriptions, FolderDescriptions};
use crate::ai_ui::AIConfigurationUI;
use eframe::egui::{self, Grid, ScrollArea};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Receiver, Sender};

pub struct AppDataCleaner {
    // 基础字段
    is_scanning: bool,
    current_folder: Option<String>,
    folder_data: Vec<(String, u64)>,
    selected_appdata_folder: String,
    tx: Option<Sender<(String, u64)>>,
    rx: Option<Receiver<(String, u64)>>,
    total_size: u64,

    // 界面状态字段
    show_about_window: bool,
    show_ai_config_window: bool,     // AI配置窗口显示状态
    show_prompt_editor: bool,        // Prompt编辑器显示状态 
    confirm_delete: Option<(String, bool)>,
    status: Option<String>,          // 状态信息
    current_tab: String,             // 当前选中的标签页

    // 日志相关字段
    is_logging_enabled: bool,
    previous_logging_state: bool,

    // 排序相关字段 
    sort_criterion: Option<String>,  // 排序标准:"name"或"size"
    sort_order: Option<String>,      // 排序顺序:"asc"或"desc"

    // 文件夹描述相关
    folder_descriptions: Option<FolderDescriptions>,
    yaml_error_logged: bool,
    ignored_folders: HashSet<String>,

    // 移动模块
    move_module: move_module::MoveModule,

    // 替换原有AI相关字段
    ai_ui: AIConfigurationUI,
    ai_rx: Option<Receiver<(String, String, String)>>, // 添加 AI 响应接收器
}

impl Default for AppDataCleaner {
    fn default() -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        let (ai_tx, ai_rx) = std::sync::mpsc::channel();  // 创建 AI 通信通道
        
        // 加载AI配置
        let ai_config = match AIConfig::load_from_file("folders_description.yaml") {
            Ok(config) => {
                logger::log_info("已成功加载AI配置文件");
                config
            }
            Err(_) => {
                logger::log_info("未找到配置文件，使用默认配置");
                AIConfig::default()
            }
        };

        // 创建 AIHandler 并包装在 Arc<Mutex<>> 中
        let ai_handler = Arc::new(Mutex::new(AIHandler::new(
            ai_config.clone(),
            Some(ai_tx.clone())
        )));

        let ai_ui = AIConfigurationUI::new(ai_config.clone(), ai_handler.clone());

        Self {
            // 基础字段初始化
            is_scanning: false,
            current_folder: None,
            folder_data: vec![],
            selected_appdata_folder: "Roaming".to_string(),
            tx: Some(tx),
            rx: Some(rx),
            total_size: 0,

            // 界面状态初始化
            show_about_window: false,
            show_ai_config_window: false,
            show_prompt_editor: false,
            confirm_delete: None,
            status: Some("未扫描".to_string()),
            current_tab: "主页".to_string(),  // 默认选中主页标签

            // 日志相关初始化
            is_logging_enabled: false,
            previous_logging_state: false,

            // 排序相关初始化
            sort_criterion: None,
            sort_order: None,

            // 文件夹描述相关初始化
            folder_descriptions: None,
            yaml_error_logged: false,
            ignored_folders: ignore::load_ignored_folders(),

            // 移动模块初始化
            move_module: Default::default(),

            // AI相关初始化
            ai_ui,
            ai_rx: Some(ai_rx),  // 保存 AI 响应接收器
        }
    }
}

impl AppDataCleaner {
    fn setup_custom_fonts(&self, ctx: &egui::Context) {
        use eframe::egui::{FontData, FontDefinitions};

        let mut fonts = FontDefinitions::default();

        fonts.font_data.insert(
            "custom_font".to_owned(),
            FontData::from_static(include_bytes!("../assets/SourceHanSansCN-Regular.otf")),
        );

        fonts.families.insert(
            egui::FontFamily::Proportional,
            vec!["custom_font".to_owned()],
        );
        fonts
            .families
            .insert(egui::FontFamily::Monospace, vec!["custom_font".to_owned()]);

        ctx.set_fonts(fonts);
    }

    // 抽取文件夹操作逻辑到单独的方法
    fn handle_folder_operations(&mut self, ui: &mut egui::Ui, folder: &str, size: u64) {
        // 显示文件夹名称和大小
        if self.ignored_folders.contains(folder) {
            ui.add_enabled(false, egui::Label::new(
                egui::RichText::new(folder).color(egui::Color32::GRAY),
            ));
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
        let description = self.folder_descriptions.as_ref()
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
        self.status = Some(format!("正在为 {} 生成描述...", folder));
        let folder_name = folder.to_string();
        let selected_folder = self.selected_appdata_folder.clone();
        let handler = self.ai_ui.get_handler();
        
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                if let Ok(mut handler) = handler.lock() {
                    if let Err(e) = handler.generate_single_description(folder_name.clone(), selected_folder).await {
                        logger::log_error(&format!("生成描述失败: {}", e));
                    }
                }
            });
        });
    }

    fn show_top_menu(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {  
                // 左侧标签页和选项
                ui.selectable_value(&mut self.current_tab, "主页".to_string(), "主页");
                ui.selectable_value(&mut self.current_tab, "关于".to_string(), "关于");
                ui.selectable_value(&mut self.current_tab, "AI配置".to_string(), "AI配置");
                ui.label("|"); // 添加分隔符
                ui.checkbox(&mut self.is_logging_enabled, "启用日志");

                // 添加一个弹性空间，将后面的内容推到右侧
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // 切换文件夹按钮
                    ui.menu_button("切换文件夹", |ui| {
                        for folder in ["Roaming", "Local", "LocalLow"] {
                            if ui.button(folder).clicked() {
                                self.selected_appdata_folder = folder.to_string();
                                self.folder_data.clear();
                                self.is_scanning = false;
                                self.status = Some("未扫描".to_string()); // 更新状态为 "未扫描"
                                ui.close_menu();
                            }
                        }
                    });
                    // 当前目标文件夹显示
                    ui.label(format!("当前目标: {}", self.selected_appdata_folder));
                });
            });

            ui.separator();
        });
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.setup_custom_fonts(ctx);
        
        // 初始化逻辑
        self.initialize_if_needed();

        // 处理 AI 响应，忽略不需要的变量
        if let Some(rx) = &self.ai_rx {
            while let Ok((_, folder_name, _)) = rx.try_recv() {
                // 重新加载描述文件以更新显示
                self.folder_descriptions = load_folder_descriptions("folders_description.yaml", &mut self.yaml_error_logged);
                // 更新状态
                self.status = Some(format!("已更新 {} 的描述", folder_name));
                // 强制重绘
                ctx.request_repaint();
            }
        }

        // 顶部菜单
        self.show_top_menu(ctx);

        // 主面板 - 根据当前标签页显示不同内容
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_tab.as_str() {
                "主页" => self.show_main_panel(ui),
                "关于" => about::show_about_content(ui),
                "AI配置" => self.ai_ui.draw_config_ui(ui),
                _ => self.show_main_panel(ui),
            }
        });

        // 窗口显示
        self.show_windows(ctx);
    }
}

// 将上述方法实现为 AppDataCleaner 的扩展方法
impl AppDataCleaner {
    fn initialize_if_needed(&mut self) {
        // 加载描述文件
        if self.folder_descriptions.is_none() {
            self.folder_descriptions =
                load_folder_descriptions("folders_description.yaml", &mut self.yaml_error_logged);
        }

        if self.is_logging_enabled != self.previous_logging_state {
            logger::init_logger(self.is_logging_enabled); // 初始化日志系统
            if self.is_logging_enabled {
                logger::log_info("日志系统已启用");
            } else {
                logger::log_info("日志系统已禁用");
            }
            self.previous_logging_state = self.is_logging_enabled; // 更新状态
        }

        // 删除确认弹窗逻辑
        confirmation::handle_delete_confirmation(
            &egui::Context::default(),
            &mut self.confirm_delete,
            &self.selected_appdata_folder,
            &mut self.status,
            &mut self.folder_data,
        ); // 传递 folder_data
    }

    fn show_main_panel(&mut self, ui: &mut egui::Ui) {
        // 扫描按钮和生成描述按钮放在一起
        ui.horizontal(|ui| {
            if ui.button("立即扫描").clicked() && !self.is_scanning {
                self.is_scanning = true;
                self.folder_data.clear();
                self.status = Some("扫描中...".to_string()); // 更新状态为 "扫描中..."

                let tx = self.tx.clone().unwrap();
                let folder_type = self.selected_appdata_folder.clone();

                scanner::scan_appdata(tx, &folder_type);
            }
            
            // 将"一键生成所有描述"按钮移到这里
            if ui.button("一键生成所有描述").clicked() {
                let folder_data = self.folder_data.clone();
                let selected_folder = self.selected_appdata_folder.clone();
                let handler = self.ai_ui.get_handler();
                
                self.status = Some("正在生成描述...".to_string());
                
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        if let Ok(mut handler) = handler.lock() {
                            if let Err(e) = handler.generate_all_descriptions(folder_data, selected_folder).await {
                                logger::log_error(&format!("批量生成描述失败: {}", e));
                            }
                        }
                    });
                });
            }
        });

        if let Some(rx) = &self.rx {
            while let Ok((folder, size)) = rx.try_recv() {
                // 检查是否接收到扫描完成标志
                if folder == "__SCAN_COMPLETE__" {
                    self.is_scanning = false;
                    self.status = Some("扫描完成".to_string()); // 更新状态为 "扫描完成"
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

    fn show_sort_controls(&mut self, ui: &mut egui::Ui) {
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
    }

    fn show_folder_grid(&mut self, ui: &mut egui::Ui) {
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

    fn show_windows(&mut self, ctx: &egui::Context) {
        // 关于窗口
        if self.show_about_window {
            about::show_about_window(ctx, &mut self.show_about_window);
        }

        // AI配置窗口(使用重构后的AI UI模块)
        self.ai_ui.show(ctx);

        // 移动窗口
        self.move_module.show_move_window(ctx);
    }
}

impl eframe::App for AppDataCleaner {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.update(ctx, frame);
    }
}