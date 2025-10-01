use bon::Builder;

use crate::system::value_object::file_id::FileId;

#[derive(Debug, Clone, Builder)]
#[readonly::make]
pub struct File {
    pub id: FileId,
    pub name: String,
    pub path: String,
    pub size: i64,
    pub used: bool,
}

impl File {
    pub fn update_name(&mut self, name: String) {
        self.name = name;
    }
    pub fn update_path(&mut self, path: String) {
        self.path = path;
    }
    pub fn update_size(&mut self, size: i64) {
        self.size = size;
    }
    pub fn update_used(&mut self, used: bool) {
        self.used = used;
    }
}
