use crate::ai_config::{AIConfig, AIHandler};
use eframe::egui;
use std::sync::{Arc, Mutex};

// 定义标签类型枚举
#[derive(PartialEq)]
enum ConfigTab {
    ApiSettings,
    RetrySettings,
    PromptSettings,
}

pub struct AIConfigurationUI {
    pub show_ai_config_window: bool,
    pub show_prompt_editor: bool,
    ai_config: AIConfig,
    ai_handler: Arc<Mutex<AIHandler>>,
    status: Option<String>,
    current_tab: ConfigTab,
    last_config: Option<AIConfig>,
}

impl AIConfigurationUI {
    pub fn new(ai_config: AIConfig, ai_handler: Arc<Mutex<AIHandler>>) -> Self {
        Self {
            show_ai_config_window: false,
            show_prompt_editor: false,
            ai_config: ai_config.clone(),
            ai_handler,
            status: None,
            current_tab: ConfigTab::ApiSettings,
            last_config: Some(ai_config),
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

    // 检查配置是否变化并保存
    fn check_and_save_config(&mut self) {
        let config_changed = match &self.last_config {
            Some(last) => {
                // 简单比较一些关键字段
                last.model.url != self.ai_config.model.url ||
                last.model.api_key != self.ai_config.model.api_key ||
                last.model.model != self.ai_config.model.model ||
                last.retry.attempts != self.ai_config.retry.attempts ||
                last.retry.delay != self.ai_config.retry.delay ||
                last.model.prompt != self.ai_config.model.prompt
            },
            None => true,
        };

        if config_changed {
            if let Ok(config_path) = AIConfig::get_config_path() {
                match self.ai_config.save_to_file(config_path.to_str().unwrap()) {
                    Ok(_) => {
                        self.status = Some("配置已自动保存".to_string());
                        // 更新 last_config
                        self.last_config = Some(self.ai_config.clone());
                        // 重新加载配置到 handler
                        if let Ok(mut handler) = self.ai_handler.lock() {
                            handler.update_config(self.ai_config.clone());
                        }
                    },
                    Err(e) => self.status = Some(format!("自动保存失败: {}", e)),
                }
            }
        }
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
                    if ui.button("确定").clicked() {
                        self.show_prompt_editor = false;
                        // 退出编辑器时检查并保存配置
                        self.check_and_save_config();
                    }
                    if ui.button("取消").clicked() {
                        // 如果取消，则需要恢复原来的 prompt
                        if let Some(last_config) = &self.last_config {
                            self.ai_config.model.prompt = last_config.model.prompt.clone();
                        }
                        self.show_prompt_editor = false;
                    }
                });
            });
    }

    // 修改为公共方法，可以直接在主面板上调用
    pub fn draw_config_ui(&mut self, ui: &mut egui::Ui) {
        // 创建整体垂直布局
        ui.vertical(|ui| {
            // 顶部标签栏
            ui.horizontal(|ui| {
                if ui.selectable_label(self.current_tab == ConfigTab::ApiSettings, "API设置").clicked() {
                    self.current_tab = ConfigTab::ApiSettings;
                }
                if ui.selectable_label(self.current_tab == ConfigTab::RetrySettings, "重试设置").clicked() {
                    self.current_tab = ConfigTab::RetrySettings;
                }
                if ui.selectable_label(self.current_tab == ConfigTab::PromptSettings, "Prompt设置").clicked() {
                    self.current_tab = ConfigTab::PromptSettings;
                }
            });
            
            ui.separator();
            
            // 中间部分：根据选中的标签显示对应的内容区域
            ui.group(|ui| {
                match self.current_tab {
                    ConfigTab::ApiSettings => self.draw_basic_settings(ui),
                    ConfigTab::RetrySettings => self.draw_retry_settings(ui),
                    ConfigTab::PromptSettings => self.draw_prompt_settings(ui),
                }
            });
            
            ui.separator();
            
            // 显示状态信息
            if let Some(status) = &self.status {
                ui.label(status);
            }
            
            // 检查配置是否变化并自动保存
            self.check_and_save_config();
        });
    }

    fn draw_basic_settings(&mut self, ui: &mut egui::Ui) {
        ui.heading("API设置");
        
        let mut changed = false;
        
        // API 配置
        ui.horizontal(|ui| {
            ui.label("API地址:");
            if ui.add(egui::TextEdit::singleline(&mut self.ai_config.model.url)).changed() {
                changed = true;
            }
        });
        
        ui.horizontal(|ui| {
            ui.label("API密钥:");
            if ui.add(egui::TextEdit::singleline(&mut self.ai_config.model.api_key).password(false)).changed() {
                changed = true;
            }
        });
        
        ui.horizontal(|ui| {
            ui.label("模型名称:");
            if ui.add(egui::TextEdit::singleline(&mut self.ai_config.model.model)).changed() {
                changed = true;
            }
        });
        
        // 添加测试连接按钮 - 修复生命周期问题
        ui.horizontal(|ui| {
            if ui.button("测试连接").clicked() {
                // 先保存当前配置
                if changed {
                    self.check_and_save_config();
                }
                
                // 使用通道来传递连接测试结果
                let (tx, rx) = std::sync::mpsc::channel();
                let handler = self.ai_handler.clone();
                
                // 创建一个背景线程执行测试
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    let result = rt.block_on(async {
                        if let Ok(handler) = handler.lock() {
                            match handler.test_connection().await {
                                Ok(_) => Ok("连接测试成功".to_string()),
                                Err(e) => Err(format!("连接失败: {}", e)),
                            }
                        } else {
                            Err("无法获取 AI 处理器锁".to_string())
                        }
                    });
                    
                    // 发送结果到主线程
                    let _ = tx.send(result);
                });
                
                // 尝试立即接收结果（非阻塞）
                match rx.try_recv() {
                    Ok(Ok(success_msg)) => {
                        self.status = Some(success_msg);
                    }
                    Ok(Err(error_msg)) => {
                        self.status = Some(error_msg);
                    }
                    Err(_) => {
                        // 如果没有立即接收到结果，设置一个等待状态
                        self.status = Some("正在测试连接...".to_string());
                        
                        // 保存接收器供后续 UI 更新使用
                        let ctx = ui.ctx().clone();
                        std::thread::spawn(move || {
                            // 等待结果
                            if let Ok(result) = rx.recv() {
                                match result {
                                    Ok(msg) => {
                                        // 触发 UI 更新
                                        ctx.request_repaint();
                                        // 将结果通过上下文传递到下一帧 - 修复 Id 类型
                                        ctx.data_mut(|data| {
                                            data.insert_temp("ai_test_result".into(), msg);
                                        });
                                    }
                                    Err(err) => {
                                        ctx.request_repaint();
                                        ctx.data_mut(|data| {
                                            data.insert_temp("ai_test_result".into(), err);
                                        });
                                    }
                                }
                            }
                        });
                    }
                }
            }
        });
        
        // 检查是否有测试结果更新 - 修复 Id 类型
        ui.ctx().data_mut(|data| {
            if let Some(result) = data.get_temp::<String>("ai_test_result".into()) {
                self.status = Some(result.clone());
                // 使用后移除临时数据
                data.remove::<String>("ai_test_result".into());
            }
        });
    }

    fn draw_retry_settings(&mut self, ui: &mut egui::Ui) {
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
    }

    fn draw_prompt_settings(&mut self, ui: &mut egui::Ui) {
        ui.heading("Prompt设置");
        if ui.button("编辑Prompt").clicked() {
            self.show_prompt_editor = true;
        }
        ui.label("当前Prompt预览:");
        ui.label(self.ai_config.model.prompt.lines().take(3).collect::<Vec<_>>().join("\n"));
    }
}
