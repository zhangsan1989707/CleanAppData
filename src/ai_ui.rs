use crate::ai_config::{AIConfig, AIHandler};
use eframe::egui;
use std::sync::{Arc, Mutex};

pub struct AIConfigurationUI {
    pub show_ai_config_window: bool,
    pub show_prompt_editor: bool,
    ai_config: AIConfig,
    ai_handler: Arc<Mutex<AIHandler>>,
    status: Option<String>,
}

impl AIConfigurationUI {
    pub fn new(ai_config: AIConfig, ai_handler: Arc<Mutex<AIHandler>>) -> Self {
        Self {
            show_ai_config_window: false,
            show_prompt_editor: false,
            ai_config,
            ai_handler,
            status: None,
        }
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        if self.show_prompt_editor {
            self.show_prompt_editor_window(ctx);
        }
    }

    // 添加获取 handler 的方法
    pub fn get_handler(&self) -> Arc<Mutex<AIHandler>> {
        self.ai_handler.clone()
    }

    // 添加获取配置的方法
    pub fn get_config(&self) -> AIConfig {
        self.ai_config.clone()
    }

    // 添加更新状态的方法
    pub fn set_status(&mut self, status: String) {
        self.status = Some(status);
    }

    fn show_prompt_editor_window(&mut self, ctx: &egui::Context) {
        egui::Window::new("Prompt编辑器")
            .resizable(true)
            .show(ctx, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut self.ai_config.model.prompt)
                        .desired_width(f32::INFINITY)
                        .desired_rows(20)
                );
                
                ui.horizontal(|ui| {
                    if ui.button("保存").clicked() {
                        self.show_prompt_editor = false;
                    }
                    if ui.button("取消").clicked() {
                        self.show_prompt_editor = false;
                    }
                });
            });
    }

    // 修改为公共方法，可以直接在主面板上调用
    pub fn draw_config_ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("AI配置生成器");
        
        // 基本配置组
        self.draw_basic_settings(ui);
        
        // API配置组
        self.draw_api_settings(ui);
        
        // 重试配置组
        self.draw_retry_settings(ui);
        
        // Prompt设置组
        self.draw_prompt_settings(ui);
        
        // 按钮组
        self.draw_action_buttons(ui);
    }

    fn draw_basic_settings(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.heading("基本设置");
            // API 配置
            ui.horizontal(|ui| {
                ui.label("API地址:");
                ui.add(egui::TextEdit::singleline(&mut self.ai_config.model.url));
            });
            ui.horizontal(|ui| {
                ui.label("API密钥:");
                ui.add(egui::TextEdit::singleline(&mut self.ai_config.model.api_key).password(true));
            });
            ui.horizontal(|ui| {
                ui.label("模型名称:");
                ui.add(egui::TextEdit::singleline(&mut self.ai_config.model.model));
            });
        });
    }

    fn draw_api_settings(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.heading("API设置");
            ui.horizontal(|ui| {
                ui.label("API密钥:");
                ui.add(egui::TextEdit::singleline(&mut self.ai_config.model.api_key).password(true));
            });
            ui.horizontal(|ui| {
                ui.label("模型名称:");
                ui.add(egui::TextEdit::singleline(&mut self.ai_config.model.model));
            });
        });
    }

    fn draw_retry_settings(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.heading("重试设置");
            ui.horizontal(|ui| {
                ui.label("重试次数:");
                ui.add(egui::DragValue::new(&mut self.ai_config.retry.attempts)
                    .range(1..=10));
            });
            ui.horizontal(|ui| {
                ui.label("重试延迟(秒):");
                ui.add(egui::DragValue::new(&mut self.ai_config.retry.delay)
                    .range(1..=60));
            });
        });
    }

    fn draw_prompt_settings(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.heading("Prompt设置");
            if ui.button("编辑Prompt").clicked() {
                self.show_prompt_editor = true;
            }
            ui.label("当前Prompt预览:");
            ui.label(self.ai_config.model.prompt.lines().take(3).collect::<Vec<_>>().join("\n"));
        });
    }

    fn draw_action_buttons(&mut self, ui: &mut egui::Ui) {
        let status_sender = Arc::new(Mutex::new(None::<String>));
        let status_sender_clone = status_sender.clone();

        ui.horizontal(|ui| {
            if ui.button("保存配置").clicked() {
                if let Ok(config_path) = AIConfig::get_config_path() {
                    match self.ai_config.save_to_file(config_path.to_str().unwrap()) {
                        Ok(_) => {
                            self.status = Some("配置已保存".to_string());
                            // 重新加载配置到 handler
                            if let Ok(mut handler) = self.ai_handler.lock() {
                                handler.update_config(self.ai_config.clone());
                            }
                        },
                        Err(e) => self.status = Some(format!("保存失败: {}", e)),
                    }
                }
            }

            if ui.button("测试连接").clicked() {
                let handler = self.ai_handler.clone();
                
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        if let Ok(handler) = handler.lock() {
                            match handler.test_connection().await {
                                Ok(_) => {
                                    if let Ok(mut status) = status_sender_clone.lock() {
                                        *status = Some("连接测试成功".to_string());
                                    }
                                },
                                Err(e) => {
                                    if let Ok(mut status) = status_sender_clone.lock() {
                                        *status = Some(format!("连接失败: {}", e));
                                    }
                                },
                            }
                        }
                    });
                });
            }
        });

        // 更新状态
        if let Ok(status) = status_sender.lock() {
            if let Some(status_str) = status.as_ref() {
                self.status = Some(status_str.clone());
            }
        }

        if let Some(status) = &self.status {
            ui.label(status);
        }
    }
}
