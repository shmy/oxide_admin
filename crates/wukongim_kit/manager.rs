use crate::{
    error::Result,
    response::{
        common::{DeviceFlag, StatusResponse},
        user::OnlineStatus,
    },
};
use reqwest::Client;

pub struct WukongIMManager {
    base_url: String,
    client: Client,
}

impl WukongIMManager {
    pub fn new(base_url: String) -> Self {
        let client = Client::default();
        Self { base_url, client }
    }

    // https://docs.githubim.com/zh/api/user/online-status
    pub async fn user_token(
        &self,
        uid: String,
        token: String,
        device_flag: DeviceFlag,
    ) -> Result<StatusResponse> {
        let url = format!("{}/user/token", self.base_url);
        let response: StatusResponse = self
            .client
            .post(url)
            .json(&serde_json::json!({ "uid": uid, "token": token, "device_flag": device_flag }))
            .send()
            .await?
            .json()
            .await?;
        Ok(response)
    }

    // https://docs.githubim.com/zh/api/user/device-quit
    pub async fn user_device_quit(
        &self,
        uid: String,
        device_flag: DeviceFlag,
    ) -> Result<StatusResponse> {
        let url = format!("{}/user/device_quit", self.base_url);
        let response: StatusResponse = self
            .client
            .post(url)
            .json(&serde_json::json!({ "uid": uid, "device_flag": device_flag }))
            .send()
            .await?
            .json()
            .await?;
        Ok(response)
    }

    // https://docs.githubim.com/zh/api/user/online-status
    pub async fn user_onlinestatus(&self, uids: &[String]) -> Result<Vec<OnlineStatus>> {
        let url = format!("{}/user/onlinestatus", self.base_url);
        let response: Vec<OnlineStatus> = self
            .client
            .post(url)
            .json(uids)
            .send()
            .await?
            .json()
            .await?;
        Ok(response)
    }
}

mod test {
    use super::*;
    use tokio::test;

    #[test]
    async fn test_token() {
        let manager = WukongIMManager::new("http://127.0.0.1:5001".to_owned());
        let response = manager
            .user_token("admin3".to_owned(), "test".to_owned(), DeviceFlag::Desktop)
            .await
            .unwrap();
        println!("{:?}", response);
    }

    #[test]
    async fn test_device_quit() {
        let manager = WukongIMManager::new("http://127.0.0.1:5001".to_owned());
        let response = manager
            .user_device_quit("admin2".to_owned(), DeviceFlag::Web)
            .await
            .unwrap();
        println!("{:?}", response);
    }

    #[test]
    async fn test_onlinestatus() {
        let manager = WukongIMManager::new("http://127.0.0.1:5001".to_owned());
        let onlinestatus = manager
            .user_onlinestatus(&vec!["admin2".to_owned(), "test".to_owned()])
            .await
            .unwrap();
        println!("{:?}", onlinestatus);
    }
}
