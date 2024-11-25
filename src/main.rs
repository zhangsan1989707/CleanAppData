mod about;
mod delete;
mod scanner;
mod ui;
mod utils;

use ui::AppDataCleaner;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "AppData Cleaner",
        options,
        Box::new(|_| Ok(Box::new(AppDataCleaner::default()))),
    )
}
