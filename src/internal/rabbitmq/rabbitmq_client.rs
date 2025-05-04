use crate::services::ConfigRabbitmq;
use lapin::{Channel, Connection, ConnectionProperties, Consumer, options::*, types::FieldTable};
use std::sync::Arc;

pub struct RabbitMqClient {
    connection: Arc<Connection>,
    channel: Channel,
    queue_name: String,
}

impl RabbitMqClient {
    pub async fn new(config: &ConfigRabbitmq, queue_name: String) -> Result<Self, lapin::Error> {
        let addr = format!(
            "amqp://{}:{}@{}:{}",
            config.username, config.password, config.host, config.port
        );
        let connection = Connection::connect(&addr, ConnectionProperties::default()).await?;
        let channel = connection.create_channel().await?;

        // Declare the queue to ensure it exists
        channel
            .queue_declare(
                &queue_name,
                QueueDeclareOptions::default(),
                FieldTable::default(),
            )
            .await?;

        Ok(RabbitMqClient {
            connection: Arc::new(connection),
            channel,
            queue_name,
        })
    }

    /// Publish a message to the queue
    pub async fn publish(&self, data: &[u8]) -> Result<(), lapin::Error> {
        self.channel
            .basic_publish(
                "",
                &self.queue_name,
                BasicPublishOptions::default(),
                data,
                lapin::BasicProperties::default(),
            )
            .await?
            .await?; // Wait for confirmation
        Ok(())
    }

    /// Consume messages from the queue
    pub async fn consume(&self) -> Result<Consumer, lapin::Error> {
        let channel = self.connection.create_channel().await?;
        channel
            .basic_consume(
                &self.queue_name,
                "",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await
    }
}
