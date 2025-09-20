use std::path::PathBuf;

pub type WorkspaceRef = std::sync::Arc<Workspace>;

#[derive(Debug, Clone)]
pub struct Workspace {
    data_dir: PathBuf,
    upload_dir: PathBuf,
    temp_dir: PathBuf,
    log_dir: PathBuf,
}

impl Workspace {
    pub fn data_dir(&self) -> &PathBuf {
        &self.data_dir
    }

    pub fn upload_dir(&self) -> &PathBuf {
        &self.upload_dir
    }

    pub fn temp_dir(&self) -> &PathBuf {
        &self.temp_dir
    }

    pub fn log_dir(&self) -> &PathBuf {
        &self.log_dir
    }
}

impl Default for Workspace {
    fn default() -> Self {
        let current_exe = std::env::current_exe().expect("get current executable path");
        let current = current_exe.parent().expect("get current executable dir");
        let data_dir = current.join("data");
        std::fs::create_dir_all(&data_dir).expect("create data dir");
        let upload_dir = data_dir.join("uploads");
        std::fs::create_dir_all(&upload_dir).expect("create upload dir");
        let temp_dir = data_dir.join("temp");
        std::fs::create_dir_all(&temp_dir).expect("create upload dir");
        let log_dir = data_dir.join("logs");
        std::fs::create_dir_all(&log_dir).expect("create upload dir");
        Self {
            data_dir,
            upload_dir,
            temp_dir,
            log_dir,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dirs_created() {
        let workspace = Workspace::default();

        assert!(workspace.data_dir().exists() && workspace.data_dir().is_dir());
        assert!(workspace.temp_dir().exists() && workspace.temp_dir().is_dir());
        assert!(workspace.upload_dir().exists() && workspace.upload_dir().is_dir());
        assert!(workspace.log_dir().exists() && workspace.log_dir().is_dir());
    }
}
