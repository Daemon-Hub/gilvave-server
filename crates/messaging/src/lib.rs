use iggy::prelude::*;
use serde::Serialize;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::mpsc;

use gilvave_settings::settings;

/// Клиент Apache Iggy для высокопроизводительного потокового обмена сообщениями.
#[derive(Clone)]
pub struct IggyBroker {
    client: Arc<IggyClient>,
    stream_name: String,
    topic_name: String,
}

// Псевдоним для полной обратной совместимости с существующим кодом Gateway
pub type RabbitClient = IggyBroker;

impl IggyBroker {
    pub async fn new(_node_id: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let (server_addr, username, password) = parse_iggy_config(settings!().iggy_url);

        tracing::info!(
            "[Apache Iggy] Connecting to broker at {} (user: {})",
            server_addr,
            username
        );

        let client = IggyClientBuilder::new()
            .with_tcp()
            .with_server_address(server_addr)
            .build()?;

        client.connect().await?;
        client.login_user(&username, &password).await?;

        let stream_name = "messages".to_string();
        let topic_name = "gateway_events".to_string();

        // Гарантируем наличие стрима
        if let Err(e) = client.create_stream(&stream_name).await {
            tracing::debug!("[Apache Iggy] Stream '{stream_name}' already exists or: {e}");
        }

        let stream_ident = Identifier::named(&stream_name)?;

        // Гарантируем наличие топика
        if let Err(e) = client
            .create_topic(
                &stream_ident,
                &topic_name,
                &TopicCreateOptions {
                    partitions_count: Some(1),
                    message_expiry: Some(IggyExpiry::NeverExpire),
                    ..Default::default()
                },
            )
            .await
        {
            tracing::debug!("[Apache Iggy] Topic '{topic_name}' already exists or: {e}");
        }

        tracing::info!(
            "[Apache Iggy] Successfully initialized broker (Stream: '{stream_name}', Topic: '{topic_name}')"
        );

        Ok(Self {
            client: Arc::new(client),
            stream_name,
            topic_name,
        })
    }

    /// Возвращает ссылку на внутренний IggyClient (сохранено имя метода get_channel для совместимости)
    pub fn get_channel(&self) -> Arc<IggyClient> {
        Arc::clone(&self.client)
    }

    /// Возвращает ссылку на внутренний IggyClient
    pub fn get_client(&self) -> Arc<IggyClient> {
        Arc::clone(&self.client)
    }

    /// Публикует сериализованное событие в топик Iggy
    pub async fn publish<T: Serialize>(
        &self,
        payload: &T,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let json_str = serde_json::to_string(payload)?;
        let stream = Identifier::named(&self.stream_name)?;
        let topic = Identifier::named(&self.topic_name)?;
        let partitioning = Partitioning::balanced();

        let message = IggyMessage::from_str(&json_str)?;
        let mut messages = vec![message];

        self.client
            .send_messages(&stream, &topic, &partitioning, &mut messages)
            .await?;

        Ok(())
    }
}

/// Запускает фоновый консьюмер сообщений из топика Apache Iggy и передаёт их в mpsc-канал.
pub async fn start_consumer(client: Arc<IggyClient>, consumer_name: &str) -> mpsc::Receiver<String> {
    let (tx, rx) = mpsc::channel::<String>(10000); // Буфер на 10к сообщений
    let consumer_name = consumer_name.to_string();

    let stream = Identifier::named("messages").expect("Invalid stream name");
    let topic = Identifier::named("gateway_events").expect("Invalid topic name");

    // Создаём персональную consumer group для этого экземпляра ноды (аналог fanout в RabbitMQ)
    client
        .create_consumer_group(&stream, &topic, &consumer_name)
        .await
        .ok();

    if let Ok(group_ident) = Identifier::named(&consumer_name) {
        client
            .join_consumer_group(&stream, &topic, &group_ident)
            .await
            .ok();
    }

    let group_ident = Identifier::named(&consumer_name).expect("Invalid group identifier");
    let consumer = Consumer::group(group_ident);

    // Фоновая таска, читающая сообщения из Iggy и отправляющая их в mpsc-канал
    tokio::spawn(async move {
        let stream = Identifier::named("messages").unwrap();
        let topic = Identifier::named("gateway_events").unwrap();
        let polling_strategy = PollingStrategy::next();

        tracing::info!("[Apache Iggy] Started consuming events for node group: {consumer_name}");

        loop {
            match client
                .poll_messages(&stream, &topic, None, &consumer, &polling_strategy, 50, true)
                .await
            {
                Ok(polled) => {
                    if polled.messages.is_empty() {
                        tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
                        continue;
                    }

                    for msg in polled.messages {
                        match std::str::from_utf8(&msg.payload) {
                            Ok(data) => {
                                if tx.send(data.to_string()).await.is_err() {
                                    tracing::info!(
                                        "[Apache Iggy] Consumer channel closed, stopping worker"
                                    );
                                    return;
                                }
                            }
                            Err(_) => tracing::error!("[Apache Iggy] Invalid UTF-8 received"),
                        }
                    }
                }
                Err(err) => {
                    tracing::error!("[Apache Iggy] Error polling messages: {err:?}");
                    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                }
            }
        }
    });

    rx
}

/// Вспомогательный парсер настроек подключения Iggy
fn parse_iggy_config(url: &str) -> (String, String, String) {
    let default_addr = "127.0.0.1:8090".to_string();
    let default_user = "iggy".to_string();
    let default_pass = "iggy_pass".to_string();

    let stripped = url.strip_prefix("iggy://").unwrap_or(url);
    if stripped.is_empty() {
        return (default_addr, default_user, default_pass);
    }

    if let Some((auth, host_port)) = stripped.split_once('@') {
        let (user, pass) = auth
            .split_once(':')
            .unwrap_or((auth, default_pass.as_str()));
        (host_port.to_string(), user.to_string(), pass.to_string())
    } else {
        (stripped.to_string(), default_user, default_pass)
    }
}

// ==============================================================================
// RabbitMQ Implementation (Deprecated / Replaced by Apache Iggy)
// ==============================================================================
// use futures::stream::StreamExt;
// use lapin::{
//     BasicProperties, Channel, Connection, ConnectionProperties, ExchangeKind,
//     options::{
//         BasicAckOptions, BasicConsumeOptions, BasicPublishOptions, ExchangeDeclareOptions,
//         QueueBindOptions, QueueDeclareOptions,
//     },
//     types::{FieldTable, ShortString},
// };
//
// #[derive(Clone)]
// pub struct RabbitClient {
//     channel: Arc<Channel>,
// }
//
// impl RabbitClient {
//     pub async fn new(node_id: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
//         let conn =
//             Connection::connect(settings!().rmq_url, ConnectionProperties::default()).await?;
//
//         let channel = conn.create_channel().await.unwrap();
//
//         channel
//             .exchange_declare(
//                 ShortString::from("messages"),
//                 ExchangeKind::Fanout,
//                 ExchangeDeclareOptions::default(),
//                 FieldTable::default(),
//             )
//             .await?;
//
//         let queue_name = ShortString::from(format!("gateway_events_{node_id}"));
//         channel
//             .queue_declare(
//                 queue_name.clone(),
//                 QueueDeclareOptions {
//                     auto_delete: true,
//                     ..Default::default()
//                 },
//                 FieldTable::default(),
//             )
//             .await?;
//
//         channel
//             .queue_bind(
//                 queue_name,
//                 ShortString::from("messages"),
//                 ShortString::from(""),
//                 QueueBindOptions::default(),
//                 FieldTable::default(),
//             )
//             .await?;
//
//         Ok(Self {
//             channel: Arc::new(channel),
//         })
//     }
//
//     pub fn get_channel(&self) -> Arc<Channel> {
//         Arc::clone(&self.channel)
//     }
//
//     pub async fn publish<T: Serialize>(
//         &self,
//         payload: &T,
//     ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
//         let data = serde_json::to_vec(payload)?;
//
//         self.get_channel()
//             .basic_publish(
//                 ShortString::from("messages"),
//                 ShortString::from(""),
//                 BasicPublishOptions::default(),
//                 &data,
//                 BasicProperties::default().with_delivery_mode(2),
//             )
//             .await?
//             .await?;
//
//         Ok(())
//     }
// }
//
// pub async fn start_consumer(channel: Arc<Channel>, queue_name: &str) -> mpsc::Receiver<String> {
//     let (tx, rx) = mpsc::channel::<String>(10000); // Буфер на 10к сообщений
//
//     let mut consumer = channel
//         .basic_consume(
//             ShortString::from(queue_name),
//             ShortString::from("consumer_tag"),
//             BasicConsumeOptions::default(),
//             FieldTable::default(),
//         )
//         .await
//         .unwrap();
//
//     tokio::spawn(async move {
//         while let Some(delivery_result) = consumer.next().await {
//             match delivery_result {
//                 Ok(delivery) => {
//                     match std::str::from_utf8(&delivery.data) {
//                         Ok(data) => {
//                             if tx.send(data.to_string()).await.is_err() {
//                                 break;
//                             }
//                         }
//                         Err(_) => tracing::error!("[RabbitMQ] Invalid UTF-8 received"),
//                     }
//                     delivery.ack(BasicAckOptions::default()).await.ok();
//                 }
//                 Err(error) => tracing::error!("[RabbitMQ] Error in consumer: {:?}", error),
//             }
//         }
//     });
//
//     rx
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "requires running Apache Iggy instance"]
    async fn test_broker_connect() {
        gilvave_settings::setup_settings();
        let broker = IggyBroker::new("test_node").await.expect("Failed to init IggyBroker");
        assert_eq!(broker.stream_name, "messages");
    }
}

