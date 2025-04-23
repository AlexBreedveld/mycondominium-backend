use lapin::Connection;
use crate::internal::config::model::ConfigRabbitmq;
pub struct RabbitMQClient {
    pub connection: Connection,
    pub channel: lapin::Channel,
}

/*
impl RabbitMQClient {
    pub async fn new(config: &ConfigRabbitmq) -> Result<Self, lapin::Error> {
        let connection = establish_connection_rabbitmq(config).await?;
        let channel = connection.create_channel(None).await?;
        Ok(RabbitMQClient { connection, channel })
    }

    pub async fn publish_message(&self, queue: &str, message: &[u8]) -> Result<(), lapin::Error> {
        self.channel.queue_declare(queue, QueueDeclareOptions::default(), FieldTable::new()).await?;
        self.channel.basic_publish(
            "",
            queue,
            BasicPublishOptions::default(),
            message,
            lapin::protocol::BasicProperties::default(),
        ).await?;
        Ok(())
    }

    pub async fn consume_messages<F>(&self, queue: &str, consumer_tag: &str, mut callback: F) -> Result<(), lapin::Error>
    where
        F: FnMut(&[u8]) -> () + Send + 'static,
    {
        self.channel.queue_declare(queue, QueueDeclareOptions::default(), FieldTable::new()).await?;
        let mut consumer = self.channel.basic_consume(
            queue,
            consumer_tag,
            BasicConsumeOptions::default(),
            FieldTable::new(),
        ).await?;
        
        tokio::spawn(async move {
            while let Some(delivery) = consumer.recv().await {
                callback(&delivery.data);
                delivery.ack(BasicAckOptions::default()).await.unwrap();
            }
        });
        Ok(())
    }
}
    */