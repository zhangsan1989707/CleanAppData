use eframe::egui;

pub fn show_confirmation(ctx: &egui::Context, message: &str) -> Option<bool> {
    let mut result = None;

    egui::Window::new("确认操作")
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.label(message);

            ui.horizontal(|ui| {
                if ui.button("确认").clicked() {
                    result = Some(true);
                }
                if ui.button("取消").clicked() {
                    result = Some(false);
                }
            });
        });

    result
}
