extern crate core;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use serde::Serialize;
use tokio::task::JoinSet;
use tokio::time;

use crate::device::{HPSwitch, OPNSense, UniFiController};

mod config;
mod device;
mod mqtt_service;

#[derive(Serialize)]
pub struct DeviceLocation {
    ipv4: String,
    ipv6: Vec<String>,
    device_mac: String,
    remote_ip: String,
    remote_mac: String,
    location: String,
}

pub enum DeviceTypes {
    HPSwitch(HPSwitch),
    UniFiController(UniFiController),
    OPNSense(OPNSense),
}

#[async_trait]
pub trait Pollable {
    async fn poll_device(&self) -> Vec<DeviceLocation>;
}

impl DeviceTypes {
    async fn get_device_locations(&self) -> Vec<DeviceLocation> {
        match self {
            DeviceTypes::UniFiController(unifi_controller) => {
                unifi_controller.poll_device().await
            }
            DeviceTypes::HPSwitch(hp_switch) => {
                hp_switch.poll_device().await
            }
            DeviceTypes::OPNSense(opnsense) => {
                opnsense.poll_device().await
            }
        }
    }
}

struct MqttPublishData {
    changed: bool,
    clients: Vec<DeviceLocation>,
}

#[tokio::main]
async fn main() {
    let (devices, mut mqtt_service, scan_interval) = config::build_config();

    let data_changed = Arc::new(
        Mutex::new(
            MqttPublishData {
                changed: false,
                clients: Vec::new(),
            }
        )
    );
    let data_changed_ref = data_changed.clone();

    let mut threads = JoinSet::new();
    threads.spawn(async move {
        loop {
            println!("Polling devices...");

            let mut total_clients: HashMap<String, DeviceLocation> = HashMap::new();
            for device in devices.iter() {
                let clients = time::timeout(
                    Duration::from_millis(5000),
                    device.get_device_locations(),
                ).await;

                if clients.is_ok() {
                    let clients = clients.unwrap();

                    for mut client in clients {
                        let old_client = total_clients.get(&client.device_mac);
                        if old_client.is_some() {
                            let old_client = old_client.unwrap();
                            if client.ipv4.is_empty() && !old_client.ipv4.is_empty() {
                                client.ipv4 = old_client.ipv4.to_string();
                            }
                        }

                        total_clients.insert(client.device_mac.to_string(), client);
                    }
                }
            }

            let clients = fill_missing_information(total_clients)
                .into_iter()
                .filter(|client| !client.remote_mac.is_empty())
                .collect();
            {
                let mut data_changed = data_changed_ref.lock().unwrap();
                data_changed.changed = true;
                data_changed.clients = clients;
            }

            time::sleep(Duration::from_secs(scan_interval)).await;
        }
    });

    threads.spawn(async move {
        loop {
            let text: Option<String>;
            {
                let mut data_changed = data_changed.lock().unwrap();
                if data_changed.changed {
                    data_changed.changed = false;

                    text = Some(serde_json::to_string(&data_changed.clients).unwrap());
                } else {
                    text = None;
                }
            }

            if text.is_some() {
                println!("Publishing mqtt data");
                mqtt_service.publish(text.unwrap()).await;
            }

            mqtt_service.process_packets().await;
        }
    });

    while let Some(_) = threads.join_next().await {}
}

fn fill_missing_information(mut clients: HashMap<String, DeviceLocation>) -> Vec<DeviceLocation> {
    let mut mac_to_ip: HashMap<String, String> = HashMap::with_capacity(clients.len());
    let mut ip_to_mac: HashMap<String, String> = HashMap::with_capacity(clients.len());

    for (mac, client) in clients.iter() {
        mac_to_ip.insert(mac.to_string(), client.ipv4.to_string());
        ip_to_mac.insert(client.ipv4.to_string(), mac.to_string());
    }

    for (_, client) in clients.iter_mut() {
        if client.remote_ip.is_empty() {
            client.remote_ip = mac_to_ip.get(&client.remote_mac).unwrap_or(&String::new()).to_string();
        }

        if client.remote_mac.is_empty() {
            client.remote_mac = ip_to_mac.get(&client.remote_ip).unwrap_or(&String::new()).to_string();
        }
    }

    clients.into_values().collect()
}