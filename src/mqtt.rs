use crate::*;
use core::option::Option;
use rust_mqtt::{
    client::{client::MqttClient, client_config::ClientConfig},
    utils::rng_generator::CountingRng,
};

pub struct MqttConnection<'a> {
    client: Option<MqttClient<'a, TcpSocket<'a>, 5, CountingRng>>,
}

impl<'a> MqttConnection<'a> {
    pub async fn new(
        socket: TcpSocket<'a>,
        rbuf: &'a mut [u8],
        rlen: usize,
        wbuf: &'a mut [u8],
        wlen: usize,
    ) -> MqttConnection<'a> {
        let mut config = ClientConfig::new(
            rust_mqtt::client::client_config::MqttVersion::MQTTv5,
            CountingRng(20000),
        );

        config.add_max_subscribe_qos(rust_mqtt::packet::v5::publish_packet::QualityOfService::QoS1);

        config.add_client_id("gstukelj");
        config.max_packet_size = 100;

        let mut client = MqttClient::<_, 5, _>::new(socket, wbuf, wlen, rbuf, rlen, config);

        match client.connect_to_broker().await {
            Ok(_) => MqttConnection::<'a> {
                client: Some(client),
            },
            Err(e) => {
                println!("failed to connect to broker!: {:?}", e);
                MqttConnection::<'a> { client: None }
            }
        }
    }

    pub async fn send_temp(&mut self, msg: &str) {
        let _ = self
            .client
            .as_mut()
            .unwrap()
            .send_message(
                "temperature/1",
                msg.as_bytes(),
                rust_mqtt::packet::v5::publish_packet::QualityOfService::QoS1,
                true,
            )
            .await;
    }

    pub async fn subscribe(&mut self, topic: &str) {
        let _ = self
            .client
            .as_mut()
            .unwrap()
            .subscribe_to_topic(topic)
            .await;
    }

    pub async fn recv_msg(&mut self) -> Option<(&str, &[u8])> {
        self.client.as_mut().unwrap().receive_message().await.ok()
    }
}
