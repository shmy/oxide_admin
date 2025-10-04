use bon::Builder;

use crate::organization::value_object::department_id::DepartmentId;

#[derive(Debug, Clone, Builder)]
#[readonly::make]
pub struct Department {
    pub id: DepartmentId,
    pub name: String,
    pub code: String,
    pub parent_code: Option<String>,
}

impl Department {
    pub fn update_name(&mut self, name: String) {
        self.name = name;
    }
    pub fn update_code(&mut self, code: String) {
        self.code = code;
    }
    pub fn update_parent_code(&mut self, parent_code: Option<String>) {
        self.parent_code = parent_code;
    }
}
