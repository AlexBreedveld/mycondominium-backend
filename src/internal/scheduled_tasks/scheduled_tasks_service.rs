use crate::establish_connection_pg;
use crate::internal::rabbitmq::rabbitmq_client::RabbitMqClient;
use crate::internal::smtp::smtp_service::listen_and_send_emails;
use crate::models::auth_model::PasswordResetModel;
use crate::services::{DatabaseTrait, MyCondominiumConfig};
use diesel::prelude::*;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

pub async fn scheduled_tasks_service(conf: Arc<MyCondominiumConfig>) {
    let rabbitmq_conn_smtp =
        match RabbitMqClient::new(&conf.rabbitmq, "mycondominium_smtp".to_string()).await {
            Ok(rmq_conn) => rmq_conn,
            Err(e) => {
                log::error!("Failed to instantiate connection with RabbitMQ: {}", e);
                panic!("Failed to instantiate connection with RabbitMQ: {}", e);
            }
        };

    let conf = conf.clone();

    let smtp_config = conf.clone().smtp.clone();
    tokio::spawn(listen_and_send_emails(rabbitmq_conn_smtp, smtp_config));
    tokio::spawn(delete_expired_password_resets(conf));
}

async fn delete_expired_password_resets(conf: Arc<MyCondominiumConfig>) {
    let conn = &mut establish_connection_pg(&conf);

    loop {
        let cutoff_time = chrono::Utc::now().naive_utc() - chrono::Duration::minutes(15);

        match diesel::delete(PasswordResetModel::table())
            .filter(crate::schema::password_reset::created_at.lt(cutoff_time))
            .execute(conn)
        {
            Ok(_) => {}
            Err(e) => {
                log::error!("Failed to delete expired password resets: {}", e);
            }
        }

        sleep(Duration::from_secs(10)).await;
    }
}
