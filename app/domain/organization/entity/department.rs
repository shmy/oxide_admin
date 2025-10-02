use bon::Builder;

use crate::organization::value_object::department_id::DepartmentId;

#[derive(Debug, Clone, Builder)]
#[readonly::make]
pub struct Department {
    pub id: DepartmentId,
    pub name: String,
    pub code: String,
    pub parent_id: Option<String>,
    pub enabled: bool,
}

impl Department {
    pub fn update_name(&mut self, name: String) {
        self.name = name;
    }
    pub fn update_code(&mut self, code: String) {
        self.code = code;
    }
    pub fn update_parent_id(&mut self, parent_id: Option<String>) {
        self.parent_id = parent_id;
    }
    pub fn update_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}
