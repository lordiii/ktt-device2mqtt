use crate::config::AppConfig;
use crate::mqtt_service::MqttService;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct MqttConfig {
    host: String,
    user: String,
    password: String,
    topic: String,
}

pub(crate) fn build_mqtt(config: &AppConfig) -> MqttService {
    if config.mqtt.is_some() {
        let mqtt = config.mqtt.as_ref().unwrap();
        return MqttService::new(&mqtt.host, &mqtt.user, &mqtt.password, &mqtt.topic)
    } else {
        panic!("Invalid MQTT Config...")
    }
}