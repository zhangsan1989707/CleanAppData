use eframe::egui;

pub fn show_about_window(ctx: &egui::Context, open: &mut bool) {
    egui::Window::new("关于此软件")
        .open(open)
        .resizable(false)
        .collapsible(false)
        .show(ctx, |ui| {
            ui.heading("AppData Cleaner");
            ui.horizontal(|ui| {
                ui.label("作者: ");
                // 将作者名字包装为可点击链接
                ui.hyperlink_to("TC999", "https://github.com/TC999"); // 点击名字跳转到指定链接
            });
            // 添加可点击的链接
            ui.horizontal(|ui| {
                ui.label("源代码仓库:");
                ui.hyperlink("https://github.com/TC999/AppDataCleaner");
            });
            ui.horizontal(|ui| {
                // BUG 反馈
                ui.hyperlink_to("议题", "https://github.com/TC999/AppDataCleaner/issues");
            });
            ui.label("许可证: GPL-3.0");
            ui.label("版本: 1.0.0");
        });
}
