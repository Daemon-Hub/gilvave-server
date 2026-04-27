pub mod event;

use anyhow::Result;
use futures::stream::StreamExt;
use lapin::{
    BasicProperties, Channel, Connection, ConnectionProperties, ExchangeKind,
    options::{
        BasicAckOptions, BasicConsumeOptions, BasicPublishOptions, ExchangeDeclareOptions,
        QueueBindOptions, QueueDeclareOptions,
    },
    types::{FieldTable, ShortString},
};
use serde::Serialize;
use std::sync::Arc;

use gilvave_settings::settings;

#[derive(Clone)]
pub struct RabbitClient {
    channel: Arc<Channel>,
}

impl RabbitClient {
    pub async fn new() -> Result<Self> {
        let conn =
            Connection::connect(settings!().rmq_url, ConnectionProperties::default()).await?;

        let channel = conn.create_channel().await.unwrap();

        channel
            .exchange_declare(
                ShortString::from("messages"),
                ExchangeKind::Fanout,
                ExchangeDeclareOptions::default(),
                FieldTable::default(),
            )
            .await?;

        let queue_name = ShortString::from("gateway_events");
        channel
            .queue_declare(
                queue_name.clone(),
                QueueDeclareOptions::default(),
                FieldTable::default(),
            )
            .await?;

        channel
            .queue_bind(
                queue_name,
                ShortString::from("messages"),
                ShortString::from(""),
                QueueBindOptions::default(),
                FieldTable::default(),
            )
            .await?;

        Ok(Self {
            channel: Arc::new(channel),
        })
    }

    pub fn get_channel(&self) -> Arc<Channel> {
        Arc::clone(&self.channel)
    }

    pub async fn publish<T: Serialize>(&self, payload: &T) -> anyhow::Result<()> {
        let data = serde_json::to_vec(payload)?;

        self.get_channel()
            .basic_publish(
                ShortString::from("messages"),
                ShortString::from(""),
                BasicPublishOptions::default(),
                &data,
                BasicProperties::default().with_delivery_mode(2),
            )
            .await?
            .await?;

        Ok(())
    }
}

pub async fn start_consumer(
    channel: Arc<Channel>,
    queue_name: &str,
    consumer_tag: &str,
) -> Result<()> {
    let mut consumer = channel
        .basic_consume(
            ShortString::from(queue_name),
            ShortString::from(consumer_tag),
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    while let Some(delivery_result) = consumer.next().await {
        match delivery_result {
            Ok(delivery) => {
                let _data = std::str::from_utf8(&delivery.data)?;
                // логика ...
                delivery.ack(BasicAckOptions::default()).await?;
            }
            Err(error) => eprintln!(" [!] Error in consumer: {:?}", error),
        }
    }

    Ok(())
}
