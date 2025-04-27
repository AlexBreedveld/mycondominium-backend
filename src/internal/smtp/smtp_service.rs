use crate::internal::rabbitmq::rabbitmq_client::RabbitMqClient;
use crate::internal::smtp::smtp_client::SmtpEmailPayload;
use crate::services::ConfigSmtp;
use futures_util::StreamExt;
use lettre::transport::smtp::client::{Tls, TlsParameters};
use lettre::{
    AsyncSmtpTransport, AsyncTransport, Tokio1Executor,
    message::{Mailbox, Message, SinglePart},
    transport::smtp::authentication::Credentials,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;

pub async fn listen_and_send_emails(rabbitmq: RabbitMqClient, smtp_config: ConfigSmtp) {
    let mut consumer = rabbitmq.consume().await.expect("Failed to start consumer");

    let creds = Credentials::new(smtp_config.username.clone(), smtp_config.password.clone());

    let mailer_builder = AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&smtp_config.host)
        .port(smtp_config.port.parse::<u16>().unwrap_or_else(|_| 587))
        .credentials(creds);

    let tls_params = match TlsParameters::new(smtp_config.host.clone()) {
        Ok(params) => params,
        Err(e) => {
            log::error!("Error creating TLS parameters: {:?}", e);
            return;
        }
    };

    // Handle TLS/SSL option from config
    let tls = match smtp_config
        .security
        .as_deref()
        .map(|s| s.to_lowercase())
        .as_deref()
    {
        Some("ssl") | Some("tls") => {
            // Create TlsParameters for your server domain
            Tls::Required(tls_params)
        }
        Some("starttls") => Tls::Opportunistic(
            TlsParameters::new(smtp_config.host.clone()).expect("Error creating TLS parameters"),
        ),
        _ => Tls::Opportunistic(
            TlsParameters::new(smtp_config.host.clone()).expect("Error creating TLS parameters"),
        ),
    };

    let mailer = mailer_builder.tls(tls).build();

    loop {
        match consumer.next().await {
            Some(Ok(delivery)) => {
                log::info!("Received email from {}", delivery.routing_key);
                let payload: Result<SmtpEmailPayload, _> = serde_json::from_slice(&delivery.data);
                match payload {
                    Ok(email) => {
                        let email_result = Message::builder()
                            .from(smtp_config.from.parse::<Mailbox>().unwrap_or_else(|_| {
                                Mailbox::new(None, smtp_config.from.clone().parse().unwrap())
                            }))
                            .to(email
                                .to
                                .parse::<Mailbox>()
                                .unwrap_or_else(|_| Mailbox::new(None, email.to.parse().unwrap())))
                            .subject(email.subject)
                            .singlepart(SinglePart::plain(email.body));

                        match email_result {
                            Ok(msg) => match mailer.send(msg).await {
                                Ok(_) => {
                                    log::info!("Email sent to {}", email.to);
                                    let _ = delivery.ack(Default::default()).await;
                                }
                                Err(e) => {
                                    log::error!("Failed to send email: {:?}", e);
                                    let _ = delivery.nack(Default::default()).await;
                                }
                            },
                            Err(e) => {
                                log::error!("Failed to construct email: {:?}", e);
                                let _ = delivery.nack(Default::default()).await;
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to parse email payload: {:?}", e);
                        let _ = delivery.nack(Default::default()).await;
                    }
                }
            }
            Some(Err(e)) => {
                log::error!("RabbitMQ consumer error: {:?}", e);
            }
            None => {
                log::warn!("Consumer stream closed");
                sleep(Duration::from_secs(10)).await;
            }
        }
    }
}
