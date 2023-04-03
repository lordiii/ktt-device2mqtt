use serde::Deserialize;

use crate::config::AppConfig;
use crate::device::OPNSense;
use crate::DeviceTypes;

#[derive(Deserialize)]
pub struct OPNSenseConfig {
    ip: String,
    api_key: String,
    api_secret: String,
}

pub fn build_opnsense(config: &AppConfig, devices: &mut Vec<DeviceTypes>) {
    if config.opnsense.is_some() {
        let opnsense = config.opnsense.as_ref().unwrap();
        devices.push(
            DeviceTypes::OPNSense(
                OPNSense::new(
                    &opnsense.ip,
                    &opnsense.api_key,
                    &opnsense.api_secret,
                )
            )
        );
    } else {
        println!("OPNsense not configured...skipping");
    }
}