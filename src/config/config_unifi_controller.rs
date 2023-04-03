use std::collections::HashMap;
use serde::Deserialize;

use crate::config::{AppConfig, DeviceTypes};
use crate::device::{UniFiAP, UniFiController};

#[derive(Deserialize)]
pub struct UniFiControllerConfig {
    ip: String,
    user: String,
    password: String,
    access_points: Vec<UniFiAPConfig>,
}

#[derive(Deserialize)]
struct UniFiAPConfig {
    mac: String,
    location: String,
}

pub fn build_unifi_controllers(config: &AppConfig, devices: &mut Vec<DeviceTypes>) {
    if config.unifi_controller.is_some() {
        let controller = config.unifi_controller.as_ref().unwrap();
        let mut access_points: HashMap<String, UniFiAP> = HashMap::with_capacity(controller.access_points.len());

        for ap in controller.access_points.iter() {
            access_points.insert(ap.mac.to_string(), UniFiAP {
                mac: ap.mac.to_string(),
                location: ap.location.to_string(),
            });
        }

        devices.push(
            DeviceTypes::UniFiController(
                UniFiController::new(
                    &controller.ip,
                    &controller.user,
                    &controller.password,
                    access_points,
                )
            )
        );
    } else {
        println!("UniFi Controller not configured...skipping");
    }
}