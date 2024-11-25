use eframe::egui;

pub fn show_about_window(ctx: &egui::Context, open: &mut bool) {
    egui::Window::new("关于此软件")
        .open(open)
        .resizable(false)
        .collapsible(false)
        .show(ctx, |ui| {
            ui.heading("AppData Cleaner");
            ui.label("作者: TC999");
            ui.label("源代码仓库: https://github.com/TC999/AppDataCleaner");
            ui.label("许可证: GPL-3.0");
            ui.label("版本: 1.0.0");
        });
}
