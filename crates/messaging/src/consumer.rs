use futures::stream::StreamExt;
use lapin::{
    Connection, ConnectionProperties,
    options::*,
    types::{FieldTable, ShortString},
};

pub async fn start_consumer() -> anyhow::Result<()> {
    let conn = Connection::connect(
        env!("RABBITMQ_DEFAULT_URI"),
        ConnectionProperties::default(),
    )
    .await?;

    let channel = conn.create_channel().await?;

    channel
        .queue_declare(
            ShortString::from("user_events"),
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;

    let mut consumer = channel
        .basic_consume(
            ShortString::from("user_events"),
            ShortString::from("worker"),
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    while let Some(delivery) = consumer.next().await {
        let delivery = delivery?;

        let data = std::str::from_utf8(&delivery.data)?;
        println!("Received: {}", data);

        delivery.ack(BasicAckOptions::default()).await?;
    }

    Ok(())
}
