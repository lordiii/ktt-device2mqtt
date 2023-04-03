use std::fs;

use serde::Deserialize;

use config_hp_switch::HPSwitchConfig;

use crate::config::config_opnsense::OPNSenseConfig;
use crate::config::config_unifi_controller::UniFiControllerConfig;
use crate::config::config_mqtt::MqttConfig;
use crate::DeviceTypes;
use crate::mqtt_service::MqttService;

mod config_hp_switch;
mod config_unifi_controller;
mod config_opnsense;
mod config_mqtt;

#[derive(Deserialize)]
pub struct AppConfig {
    hp_switches: Option<Vec<HPSwitchConfig>>,
    unifi_controller: Option<UniFiControllerConfig>,
    opnsense: Option<OPNSenseConfig>,
    mqtt: Option<MqttConfig>,
    scan_interval: u64,
}

pub fn build_config() -> (Vec<DeviceTypes>, MqttService, u64) {
    let config_json = fs::read_to_string("config.json").unwrap();
    let config: AppConfig = serde_json::from_str(&config_json).unwrap();

    let mut devices: Vec<DeviceTypes> = Vec::new();

    config_opnsense::build_opnsense(&config, &mut devices);
    config_hp_switch::build_hp_switches(&config, &mut devices);
    config_unifi_controller::build_unifi_controllers(&config, &mut devices);

    let mqtt = config_mqtt::build_mqtt(&config);

    return (devices, mqtt, config.scan_interval);
}