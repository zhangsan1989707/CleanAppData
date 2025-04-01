use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

pub struct StatsLogger {
    file_path: PathBuf,
}

impl StatsLogger {
    pub fn new(file_path: PathBuf) -> Self {
        Self { file_path }
    }

    pub fn log_stats(&self, cleaned_folders_count: u64, total_cleaned_size: u64) {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.file_path)
            .expect("无法打开统计日志文件");

        writeln!(
            file,
            "已清理文件夹数量: {}\n总清理大小: {} 字节",
            cleaned_folders_count, total_cleaned_size
        )
        .expect("无法写入统计日志文件");
    }
}
