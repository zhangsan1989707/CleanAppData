use eframe::egui;

// 窗口形式显示关于内容
pub fn show_about_window(ctx: &egui::Context, open: &mut bool) {
    egui::Window::new("关于此软件")
        .open(open)
        .resizable(false)
        .collapsible(false)
        .show(ctx, |ui| {
            show_about_content(ui);
        });
}

// 面板形式显示关于内容
pub fn show_about_content(ui: &mut egui::Ui) {
    let version = env!("CARGO_PKG_VERSION");

    ui.heading("CleanAppData");
    ui.add_space(10.0);

    ui.horizontal(|ui| {
        ui.label("作者: ");
        ui.hyperlink_to("zhangsan1989707", "https://github.com/zhangsan1989707");
    });

    ui.add_space(5.0);

    ui.horizontal(|ui| {
        ui.label("源代码仓库:");
        ui.hyperlink("https://github.com/zhangsan1989707/CleanAppData.git");
    });

    ui.add_space(5.0);

    ui.horizontal(|ui| {
        ui.label("议题反馈:");
        ui.hyperlink_to("GitHub Issues", "https://github.com/zhangsan1989707/CleanAppData/issues");
    });

    ui.add_space(10.0);

    ui.label(format!("版本: {}", version));
    ui.label("许可证: GPL-3.0");

    ui.add_space(20.0);

    ui.heading("鸣谢:");
    ui.label("egui - 一个简单、快速、高度可移植的即时模式 GUI 库");
    ui.hyperlink_to("egui 官方网站", "https://github.com/emilk/egui");

    ui.add_space(10.0);
    ui.heading("贡献者:");
    ui.hyperlink_to("Xch13", "https://github.com/Xch13");
}

// 处理关于标签页面
pub fn handle_about_tab(ui: &mut egui::Ui) {
    show_about_content(ui);
}
