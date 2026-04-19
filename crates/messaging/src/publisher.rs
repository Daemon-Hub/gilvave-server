use lapin::{BasicProperties, Channel, options::BasicPublishOptions, types::ShortString};
use serde::Serialize;

pub async fn publish<T: Serialize>(
    channel: &Channel,
    exchange: &str,
    routing_key: &str,
    payload: &T,
) -> anyhow::Result<()> {
    let data = serde_json::to_vec(payload)?;

    channel
        .basic_publish(
            ShortString::from(exchange),
            ShortString::from(routing_key),
            BasicPublishOptions::default(),
            &data,
            BasicProperties::default(),
        )
        .await?
        .await?;

    Ok(())
}
