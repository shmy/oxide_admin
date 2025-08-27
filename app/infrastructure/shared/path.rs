use std::{path::PathBuf, sync::LazyLock};

static CURRENT_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let current_exe = std::env::current_exe().expect("get current executable path");
    let current = current_exe.parent().expect("get current executable dir");
    current.to_owned()
});

pub static DATA_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let dir = CURRENT_DIR.join("data");
    std::fs::create_dir_all(&dir).expect("create data dir");
    dir.to_owned()
});

pub static TEMP_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let dir = DATA_DIR.join("temp");
    std::fs::create_dir_all(&dir).expect("create temp dir");
    dir.to_owned()
});

pub static UPLOAD_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let dir = DATA_DIR.join("uploads");
    std::fs::create_dir_all(&dir).expect("create upload dir");
    dir.to_owned()
});

pub static LOG_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let dir = DATA_DIR.join("logs");
    std::fs::create_dir_all(&dir).expect("create log dir");
    dir.to_owned()
});
