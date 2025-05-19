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

    let smtp_config = conf.clone().smtp.clone();
    tokio::spawn(listen_and_send_emails(rabbitmq_conn_smtp, smtp_config));
    tokio::spawn(delete_expired_password_resets(conf.clone()));
    tokio::spawn(update_reservation_status(conf));
}

async fn delete_expired_password_resets(conf: Arc<MyCondominiumConfig>) {
    let conn = &mut establish_connection_pg(&conf);

    log::info!("Starting expired password reset cleanup service");

    loop {
        let now = chrono::Utc::now().naive_utc();
        let cutoff_time = now - chrono::Duration::minutes(15);

        match diesel::delete(PasswordResetModel::table())
            .filter(crate::schema::password_reset::created_at.lt(cutoff_time))
            .execute(conn)
        {
            Ok(count) => {
                if count > 0 {
                    log::info!(
                        "Successfully deleted {} expired password reset tokens",
                        count
                    );
                } else {
                    log::debug!("No expired password reset tokens found to delete");
                }
            }
            Err(e) => {
                log::error!("Failed to delete expired password resets: {}", e);
            }
        }

        log::debug!("Password reset cleanup complete, next run in 10 seconds");
        sleep(Duration::from_secs(10)).await;
    }
}

async fn update_reservation_status(conf: Arc<MyCondominiumConfig>) {
    use crate::models::reservation_model::*;
    use crate::schema::reservations::dsl::*;
    use chrono::Utc;
    use diesel::prelude::*;

    log::info!("Starting update reservation status service");

    let conn = &mut establish_connection_pg(&conf);

    loop {
        let now = Utc::now().naive_utc();

        let reserved_to_ongoing = reservations
            .filter(status.eq(ReservationStatus::Reserved))
            .filter(start_time.le(now))
            .filter(end_time.gt(now))
            .load::<ReservationModel>(conn);

        match reserved_to_ongoing {
            Ok(to_update) => {
                log::info!(
                    "Found {} reservations to update from Reserved to Ongoing",
                    to_update.len()
                );

                for mut reservation in to_update {
                    reservation.status = ReservationStatus::Ongoing;
                    reservation.updated_at = now;

                    match reservation.db_update(conn) {
                        Ok(_) => {
                            log::info!("Updated reservation {} status to Ongoing", reservation.id);
                        }
                        Err(e) => {
                            log::error!(
                                "Failed to update reservation {} status to Ongoing: {}",
                                reservation.id,
                                e
                            );
                        }
                    }
                }
            }
            Err(e) => {
                log::error!(
                    "Error fetching reservations to update from Reserved to Ongoing: {}",
                    e
                );
            }
        }

        let ongoing_to_finished = reservations
            .filter(status.eq(ReservationStatus::Ongoing))
            .filter(end_time.le(now))
            .load::<ReservationModel>(conn);

        match ongoing_to_finished {
            Ok(to_update) => {
                log::info!(
                    "Found {} reservations to update from Ongoing to Finished",
                    to_update.len()
                );

                for mut reservation in to_update {
                    reservation.status = ReservationStatus::Finished;
                    reservation.updated_at = now;

                    match reservation.db_update(conn) {
                        Ok(_) => {
                            log::info!("Updated reservation {} status to Finished", reservation.id);
                        }
                        Err(e) => {
                            log::error!(
                                "Failed to update reservation {} status to Finished: {}",
                                reservation.id,
                                e
                            );
                        }
                    }
                }
            }
            Err(e) => {
                log::error!(
                    "Error fetching reservations to update from Ongoing to Finished: {}",
                    e
                );
            }
        }

        let missed_to_finished = reservations
            .filter(status.eq(ReservationStatus::Reserved))
            .filter(end_time.le(now))
            .load::<ReservationModel>(conn);

        match missed_to_finished {
            Ok(to_update) => {
                log::info!(
                    "Found {} missed reservations to update from Reserved to Finished",
                    to_update.len()
                );

                for mut reservation in to_update {
                    reservation.status = ReservationStatus::Finished;
                    reservation.updated_at = now;

                    match reservation.db_update(conn) {
                        Ok(_) => {
                            log::info!(
                                "Updated missed reservation {} status to Finished",
                                reservation.id
                            );
                        }
                        Err(e) => {
                            log::error!(
                                "Failed to update missed reservation {} status to Finished: {}",
                                reservation.id,
                                e
                            );
                        }
                    }
                }
            }
            Err(e) => {
                log::error!("Error fetching missed reservations to update: {}", e);
            }
        }

        log::info!("Completed reservation status update task at {}", now);

        sleep(Duration::from_secs(10)).await;
    }
}
