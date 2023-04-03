use std::time::{Duration};

use rumqttc::{AsyncClient, EventLoop, MqttOptions, QoS, TlsConfiguration, Transport};
use tokio::time;

pub struct MqttService {
    mqtt_client: AsyncClient,
    mqtt_events: EventLoop,
    topic: String,
}

impl MqttService {
    pub fn new(host: &str, user: &str, pass: &str, topic: &str) -> MqttService {
        let mut mqtt_options = MqttOptions::parse_url(
            format!(
                "{}?client_id=rust_location_2_mqtt",
                host
            )
        ).unwrap();

        mqtt_options.set_credentials(user, pass);
        mqtt_options.set_transport(Transport::Tls(TlsConfiguration::Native));
        mqtt_options.set_keep_alive(Duration::from_secs(5));

        let (mqtt_client, mqtt_events) = AsyncClient::new(mqtt_options, 10);

        MqttService {
            mqtt_client,
            mqtt_events,
            topic: topic.to_string(),
        }
    }

    pub async fn process_packets(&mut self) {
        let _result = time::timeout(
            Duration::from_millis(2000),
            self.mqtt_events.poll(),
        ).await;
    }

    pub async fn publish(&mut self, data: String) {
        let _ = time::timeout(
            Duration::from_millis(2000),
            self.mqtt_client.publish(&self.topic, QoS::AtMostOnce, false, data),
        ).await;
    }
}