use bon::Builder;

use crate::system::value_object::access_log_id::AccessLogId;

#[derive(Debug, Clone, Builder)]
#[readonly::make]
pub struct AccessLog {
    pub id: AccessLogId,
    pub user_id: String,
    pub method: String,
    pub uri: String,
    pub user_agent: Option<String>,
    pub ip: Option<String>,
    pub ip_region: Option<String>,
    pub status: i16,
    pub elapsed: i64,
}

impl AccessLog {
    pub fn update_user_id(&mut self, user_id: String) {
        self.user_id = user_id;
    }
    pub fn update_method(&mut self, method: String) {
        self.method = method;
    }
    pub fn update_uri(&mut self, uri: String) {
        self.uri = uri;
    }
    pub fn update_user_agent(&mut self, user_agent: Option<String>) {
        self.user_agent = user_agent;
    }
    pub fn update_ip(&mut self, ip: Option<String>) {
        self.ip = ip;
    }
    pub fn update_ip_region(&mut self, ip_region: Option<String>) {
        self.ip_region = ip_region;
    }
    pub fn update_status(&mut self, status: i16) {
        self.status = status;
    }
    pub fn update_elapsed(&mut self, elapsed: i64) {
        self.elapsed = elapsed;
    }
}
