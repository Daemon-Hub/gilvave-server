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
    pub stream_name: String,
    pub topic_name: String,
    stream_ident: Identifier,
    topic_ident: Identifier,
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

        let topic_ident = Identifier::named(&topic_name)?;

        tracing::info!(
            "[Apache Iggy] Successfully initialized broker (Stream: '{stream_name}', Topic: '{topic_name}')"
        );

        Ok(Self {
            client: Arc::new(client),
            stream_name,
            topic_name,
            stream_ident,
            topic_ident,
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
        let partitioning = Partitioning::balanced();

        let message = IggyMessage::from_str(&json_str)?;
        let mut messages = [message];

        self.client
            .send_messages(
                &self.stream_ident,
                &self.topic_ident,
                &partitioning,
                &mut messages,
            )
            .await?;

        Ok(())
    }
}

/// Запускает фоновый консьюмер сообщений из топика Apache Iggy и передаёт их в mpsc-канал.
pub async fn start_consumer(
    client: Arc<IggyClient>,
    consumer_name: &str,
) -> mpsc::Receiver<String> {
    let (tx, rx) = mpsc::channel::<String>(10000); // Буфер на 10к сообщений
    let consumer_name = consumer_name.to_string();

    let stream = Identifier::named("messages").expect("Valid stream identifier");
    let topic = Identifier::named("gateway_events").expect("Valid topic identifier");

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

    let group_ident = Identifier::named(&consumer_name).expect("Valid consumer group identifier");
    let consumer = Consumer::group(group_ident);

    // Фоновая таска, читающая сообщения из Iggy и отправляющая их в mpsc-канал
    tokio::spawn(async move {
        let stream = Identifier::named("messages").expect("Valid stream identifier");
        let topic = Identifier::named("gateway_events").expect("Valid topic identifier");
        let polling_strategy = PollingStrategy::next();

        tracing::info!("[Apache Iggy] Started consuming events for node group: {consumer_name}");

        let mut idle_duration = tokio::time::Duration::from_millis(10);
        loop {
            match client
                .poll_messages(
                    &stream,
                    &topic,
                    None,
                    &consumer,
                    &polling_strategy,
                    50,
                    true,
                )
                .await
            {
                Ok(polled) => {
                    if polled.messages.is_empty() {
                        tokio::time::sleep(idle_duration).await;
                        if idle_duration < tokio::time::Duration::from_millis(50) {
                            idle_duration += tokio::time::Duration::from_millis(5);
                        }
                        continue;
                    }
                    idle_duration = tokio::time::Duration::from_millis(5);

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
                Err(e) => {
                    tracing::error!("[Apache Iggy] Error polling messages: {e}");
                    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
                }
            }
        }
    });

    rx
}

/// Вспомогательная функция парсинга URL Iggy
fn parse_iggy_config(url: &str) -> (String, String, String) {
    if let Some(rest) = url.strip_prefix("iggy://")
        && let Some((user_pass, host_port)) = rest.split_once('@')
    {
        let (username, password) = user_pass.split_once(':').unwrap_or((user_pass, ""));
        return (
            host_port.to_string(),
            username.to_string(),
            password.to_string(),
        );
    }
    (
        "127.0.0.1:8090".to_string(),
        "iggy".to_string(),
        "iggy_pass".to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "requires running Apache Iggy instance"]
    async fn test_broker_connect() {
        gilvave_settings::setup_settings();
        let broker = IggyBroker::new("test_node")
            .await
            .expect("Failed to init IggyBroker");
        assert_eq!(broker.stream_name, "messages");
    }
}
