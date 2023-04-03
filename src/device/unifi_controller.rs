use std::collections::HashMap;

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde::de::Unexpected::Str;

use crate::{DeviceLocation, Pollable};
use crate::device::{UniFiAP, UniFiController};

#[derive(Deserialize)]
struct UniFiMetaResponseData {
    rc: String,
}

#[derive(Deserialize)]
struct UniFiMetaResponse {
    meta: UniFiMetaResponseData,
}

#[derive(Deserialize)]
struct UniFiDeviceData {
    mac: String,
    ap_mac: String,
    ip: Option<String>,
}

#[derive(Deserialize)]
struct UniFiDeviceResponse {
    data: Vec<UniFiDeviceData>,
}

#[derive(Serialize)]
struct UniFiLoginData {
    username: String,
    password: String,
    remember: bool,
    strict: bool,
}

#[async_trait]
impl Pollable for UniFiController {
    async fn poll_device(&self) -> Vec<DeviceLocation> {
        let body = self.client
            .get(&self.data_url)
            .send()
            .await;

        if body.is_err() {
            println!("{}", body.unwrap_err().to_string());
            return Vec::new();
        }
        let body = body.unwrap().text().await;
        if body.is_err() {
            println!("{}", body.unwrap_err().to_string());
            return Vec::new();
        }
        let body = body.unwrap();

        let response: UniFiMetaResponse = serde_json::from_str(&body).unwrap();
        return if response.meta.rc == "error" {
            println!("failed to request devices from unifi controller...trying to log in", );

            let is_logged_in = self.login().await;
            if is_logged_in {
                self.poll_device().await
            } else {
                println!("failed to login to unifi controller...");
                Vec::new()
            }
        } else {
            let mut devices: UniFiDeviceResponse = serde_json::from_str(&body).unwrap();

            devices.data.iter_mut().map(|device| {
                let location = self.get_ap_location(&device);

                DeviceLocation {
                    ipv4: device.ip.clone().unwrap_or(String::new()),
                    ipv6: Vec::new(),
                    device_mac: device.mac.to_string(),
                    remote_mac: device.ap_mac.to_string(),
                    remote_ip: "".to_string(),
                    location,
                }
            }).collect()
        };
    }
}

impl UniFiController {
    pub fn new(ip: &str, user: &String, password: &String, access_points: HashMap<String, UniFiAP>) -> UniFiController {
        let client = Client::builder()
            .cookie_store(true)
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap();

        UniFiController {
            ip: ip.to_string(),
            data_url: format!("https://{}:8443/api/s/default/stat/sta", ip.to_string()),
            login_url: format!("https://{}:8443/api/login", ip.to_string()),
            user: user.to_string(),
            password: password.to_string(),
            client,
            access_points,
        }
    }

    async fn login(&self) -> bool {
        let login = UniFiLoginData {
            username: self.user.to_string(),
            password: self.password.to_string(),
            remember: true,
            strict: true,
        };

        let body = self.client
            .post(&self.login_url)
            .body(serde_json::to_string(&login).unwrap())
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        let response: UniFiMetaResponse = serde_json::from_str(&body).unwrap();
        return response.meta.rc != "error";
    }

    fn get_ap_location(&self, device: &UniFiDeviceData) -> String {
        let ap = self.access_points.get(&device.ap_mac);

        return if ap.is_some() {
            ap.unwrap().location.to_string()
        } else {
            String::new()
        };
    }
}