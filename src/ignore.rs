use std::collections::HashSet;
use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, Write};
//use std::path::Path;

const IGNORE_FILE: &str = "adcignore.txt";

pub fn load_ignored_folders() -> HashSet<String> {
    let mut ignored = HashSet::new();
    if let Ok(file) = fs::File::open(IGNORE_FILE) {
        for line in io::BufReader::new(file).lines().flatten() {
            ignored.insert(line);
        }
    }
    ignored
}

pub fn save_ignored_folders(ignored: &HashSet<String>) {
    if let Ok(mut file) = OpenOptions::new().write(true).create(true).truncate(true).open(IGNORE_FILE) {
        for folder in ignored {
            writeln!(file, "{}", folder).unwrap();
        }
    }
}
