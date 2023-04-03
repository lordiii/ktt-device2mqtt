use std::collections::HashMap;
use reqwest::Client;

mod hp_switch;
mod unifi_controller;
mod opnsense;

pub struct HPSwitch {
    ip: String,
    ports: Vec<String>,
    location: String,
    data_url: String,
    client: Client,
}

pub struct UniFiController {
    ip: String,
    data_url: String,
    login_url: String,
    client: Client,
    user: String,
    password: String,
    access_points: HashMap<String, UniFiAP>,
}

pub struct UniFiAP {
    pub mac: String,
    pub location: String,
}

pub struct OPNSense {
    ip: String,
    api_key: String,
    api_secret: String,
    client: Client,
    data_url: String,
}