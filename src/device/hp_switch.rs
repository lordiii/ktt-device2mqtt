use async_trait::async_trait;
use reqwest::Error;
use serde::Deserialize;

use crate::{DeviceLocation, Pollable};
use crate::device::HPSwitch;

#[derive(Deserialize)]
struct MacTable {
    mac_table_entry_element: Vec<MacTableEntry>,
}

#[derive(Deserialize)]
struct MacTableEntry {
    mac_address: String,
    port_id: String,
    vlan_id: u16,
}

#[async_trait]
impl Pollable for HPSwitch {
    async fn poll_device(&self) -> Vec<DeviceLocation> {
        let mut clients: Vec<DeviceLocation> = Vec::new();
        let result = self.get_mac_table().await.ok();

        if result.is_some() {
            let result = result.unwrap();

            for item in result.mac_table_entry_element.iter() {
                if item.vlan_id == 23 && self.ports.contains(&item.port_id) {
                    clients.push(DeviceLocation {
                        ipv4: "".to_string(),
                        ipv6: Vec::new(),
                        device_mac: self.format_hp_mac(item.mac_address.to_string()),
                        remote_mac: "".to_string(),
                        remote_ip: self.ip.to_string(),
                        location: self.location.to_string(),
                    });
                }
            }
        }

        return clients;
    }
}

impl HPSwitch {
    pub fn new(ip: &str, ports: Vec<String>, location: &str) -> HPSwitch {
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap();

        HPSwitch {
            ip: ip.to_string(),
            ports,
            location: location.to_string(),
            data_url: format!("http://{}/rest/v1/mac-table", ip),
            client,
        }
    }

    async fn get_mac_table(&self) -> Result<MacTable, Option<Error>> {
        let body = self.client
            .get(&self.data_url)
            .send()
            .await?
            .text()
            .await?;

        let mac_table: Option<MacTable> = serde_json::from_str(&body).ok();

        return if mac_table.is_some() {
            Ok(mac_table.unwrap())
        } else {
            Err(None)
        };
    }

    fn format_hp_mac(&self, mut mac: String) -> String {
        mac = mac.replace('-', "");
        mac.insert(2, ':');
        mac.insert(5, ':');
        mac.insert(8, ':');
        mac.insert(11, ':');
        mac.insert(14, ':');
        mac.to_string()
    }
}