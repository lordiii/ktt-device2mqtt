use async_trait::async_trait;
use serde::Deserialize;

use crate::{DeviceLocation, Pollable};
use crate::device::OPNSense;

#[derive(Deserialize)]
struct ArpEntry {
    mac: String,
    ip: String,
}

#[async_trait]
impl Pollable for OPNSense {
    async fn poll_device(&self) -> Vec<DeviceLocation> {
        let body = self.client
            .get(&self.data_url)
            .basic_auth(&self.api_key, Some(&self.api_secret))
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

        let mut entries: Vec<ArpEntry> = serde_json::from_str(&body).unwrap();

        entries.iter_mut().map(|entry| {
            DeviceLocation {
                ipv4: entry.ip.to_string(),
                ipv6: Vec::new(),
                device_mac: entry.mac.to_string(),
                remote_mac: String::new(),
                remote_ip: String::new(),
                location: String::new(),
            }
        }).collect()
    }
}

impl OPNSense {
    pub fn new(ip: &str, key: &str, secret: &str) -> OPNSense {
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap();

        OPNSense {
            ip: ip.to_string(),
            data_url: format!("https://{}/api/diagnostics/interface/getArp", ip.to_string()),
            api_key: key.to_string(),
            api_secret: secret.to_string(),
            client,
        }
    }
}