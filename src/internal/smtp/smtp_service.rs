use crate::internal::rabbitmq::rabbitmq_client::RabbitMqClient;
use crate::internal::smtp::smtp_client::SmtpEmailPayload;
use crate::services::ConfigSmtp;
use futures_util::StreamExt;
use lettre::transport::smtp::client::{Tls, TlsParameters};
use lettre::{
    AsyncSmtpTransport, AsyncTransport, Tokio1Executor,
    message::{Mailbox, Message, SinglePart, header},
    transport::smtp::authentication::Credentials,
};
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::time::sleep;

const MAX_RETRIES: u32 = 3;
const RETRY_DELAY_SECS: u64 = 5;

#[derive(Error, Debug)]
pub enum EmailError {
    #[error("SMTP error: {0}")]
    SmtpError(#[from] lettre::transport::smtp::Error),
    #[error("Address parse error: {0}")]
    AddressError(#[from] lettre::address::AddressError),
    #[error("Message building error: {0}")]
    MessageBuildError(#[from] lettre::error::Error),
    #[error("Custom error: {0}")]
    Custom(String),
}

pub async fn listen_and_send_emails(rabbitmq: RabbitMqClient, smtp_config: ConfigSmtp) {
    let mut consumer = match rabbitmq.consume().await {
        Ok(consumer) => consumer,
        Err(e) => {
            log::error!("Failed to start consumer: {:?}", e);
            return;
        }
    };

    let mailer = create_mailer(&smtp_config).await;
    let mailer = match mailer {
        Ok(mailer) => Arc::new(mailer),
        Err(e) => {
            log::error!("Failed to create mailer: {:?}", e);
            return;
        }
    };

    let smtp_config = Arc::new(smtp_config);

    loop {
        match consumer.next().await {
            Some(Ok(delivery)) => {
                log::info!("Received email from {}", delivery.routing_key);
                let payload: Result<SmtpEmailPayload, _> = serde_json::from_slice(&delivery.data);

                match payload {
                    Ok(email) => {
                        let mailer = Arc::clone(&mailer);
                        let smtp_config = Arc::clone(&smtp_config);
                        let delivery_clone = delivery.clone();

                        tokio::spawn(async move {
                            let result = process_email(&email, &mailer, &smtp_config).await;
                            match result {
                                Ok(_) => {
                                    log::info!("Email sent successfully to {}", email.to);
                                    if let Err(e) = delivery_clone.ack(Default::default()).await {
                                        log::error!("Failed to acknowledge message: {:?}", e);
                                    }
                                }
                                Err(e) => {
                                    log::error!("Failed to process email: {:?}", e);
                                    if let Err(e) = delivery_clone.nack(Default::default()).await {
                                        log::error!(
                                            "Failed to negative acknowledge message: {:?}",
                                            e
                                        );
                                    }
                                }
                            }
                        });
                    }
                    Err(e) => {
                        log::error!("Failed to parse email payload: {:?}", e);
                        if let Err(e) = delivery.nack(Default::default()).await {
                            log::error!("Failed to negative acknowledge message: {:?}", e);
                        }
                    }
                }
            }
            Some(Err(e)) => {
                log::error!("RabbitMQ consumer error: {:?}", e);
                sleep(Duration::from_secs(RETRY_DELAY_SECS)).await;
            }
            None => {
                log::warn!("Consumer stream closed, attempting to reconnect...");
                sleep(Duration::from_secs(10)).await;

                match rabbitmq.consume().await {
                    Ok(new_consumer) => {
                        consumer = new_consumer;
                        log::info!("Successfully reconnected to RabbitMQ");
                    }
                    Err(e) => {
                        log::error!("Failed to reconnect to RabbitMQ: {:?}", e);
                        sleep(Duration::from_secs(10)).await;
                    }
                }
            }
        }
    }
}

async fn create_mailer(
    smtp_config: &ConfigSmtp,
) -> Result<AsyncSmtpTransport<Tokio1Executor>, EmailError> {
    let creds = Credentials::new(smtp_config.username.clone(), smtp_config.password.clone());
    let port = smtp_config.port.parse::<u16>().unwrap_or(587);

    let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(&format!("{}", smtp_config.host))?
        .port(port)
        .credentials(creds)
        .timeout(Some(Duration::from_secs(30)))
        .build();

    Ok(mailer)
}

async fn process_email(
    email: &SmtpEmailPayload,
    mailer: &AsyncSmtpTransport<Tokio1Executor>,
    smtp_config: &ConfigSmtp,
) -> Result<(), EmailError> {
    let mut retry_count = 0;

    let from_mailbox = smtp_config
        .from
        .parse::<Mailbox>()
        .map_err(|e| EmailError::Custom(format!("Invalid 'from' address: {}", e)))?;

    let to_mailbox = email
        .to
        .parse::<Mailbox>()
        .map_err(|e| EmailError::Custom(format!("Invalid 'to' address: {}", e)))?;

    let email_builder = Message::builder()
        .from(from_mailbox)
        .to(to_mailbox)
        .subject(&email.subject)
        .header(header::ContentType::TEXT_PLAIN)
        .singlepart(SinglePart::plain(email.body.clone()))?;

    loop {
        match mailer.send(email_builder.clone()).await {
            Ok(_) => return Ok(()),
            Err(e) => {
                retry_count += 1;
                if retry_count >= MAX_RETRIES {
                    return Err(EmailError::Custom(format!(
                        "Failed to send email after {} retries: {:?}",
                        MAX_RETRIES, e
                    )));
                }
                log::warn!(
                    "Failed to send email (attempt {}/{}): {:?}",
                    retry_count,
                    MAX_RETRIES,
                    e
                );
                sleep(Duration::from_secs(RETRY_DELAY_SECS)).await;
            }
        }
    }
}
