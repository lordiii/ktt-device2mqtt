use std::str::FromStr;

use regex::Regex;
use serde::Deserialize;

use crate::config::{AppConfig, DeviceTypes};
use crate::device::HPSwitch;

#[derive(Deserialize)]
pub struct HPSwitchConfig {
    ip: String,
    ports: String,
    location: String,
}

struct PortRange {
    from: u8,
    to: u8,
}

pub fn build_hp_switches(config: &AppConfig, devices: &mut Vec<DeviceTypes>) {
    let ip_regex = Regex::new(r"^[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}").unwrap();

    if config.hp_switches.is_some() {
        let switches = config.hp_switches.as_ref().unwrap();
        for switch_config in switches.iter() {
            if !ip_regex.is_match(&switch_config.ip) {
                println!("{} is not a valid IPv4 Address", &switch_config.ip);
                continue;
            }

            let switch = HPSwitch::new(
                &switch_config.ip,
                get_port_list(switch_config),
                &switch_config.location,
            );

            devices.push(DeviceTypes::HPSwitch(switch));
        }
    } else {
        println!("No HP Switches configured...skipping")
    }
}

fn get_port_list(switch_config: &HPSwitchConfig) -> Vec<String> {
    let mut ports = Vec::new();
    for port in switch_config.ports.split(',') {
        let range = generate_port_range(port);
        if range.is_some() {
            let range = range.unwrap();

            for port in range.from..range.to {
                ports.push(port.to_string());
            }
        }
    }

    return ports;
}

fn generate_port_range(port_str: &str) -> Option<PortRange> {
    let port_regex: Regex = Regex::new(r"^[0-9]{1,2}$").unwrap();
    let range_regex: Regex = Regex::new(r"^[0-9]{1,2}-[0-9]{1,2}$").unwrap();

    let port_str = port_str.trim();

    if range_regex.is_match(port_str) {
        let range: Vec<&str> = port_str.split('-').collect();

        if (&range).len() == 2 {
            let from = u8::from_str(range[0]);
            let to = u8::from_str(range[1]);

            if from.is_ok() && to.is_ok() {
                return Some(PortRange {
                    from: from.unwrap(),
                    to: to.unwrap() + 1,
                });
            }
        }
    } else if port_regex.is_match(port_str) {
        let port = u8::from_str(port_str);

        if port.is_ok() {
            let port = port.unwrap();
            return Some(PortRange {
                from: port,
                to: port,
            });
        }
    }

    println!("Invalid Port Configuration: {}", port_str);
    None
}