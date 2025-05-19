use super::*;
use chrono::Datelike;
use std::time::Duration;
use tokio::time::sleep;

#[utoipa::path(
    post,
    tag = "Authentication",
    path = "/request_password_reset",
    request_body = auth_model::PasswordResetRequestModel,
    responses (
        (status = 200, description = "Password Reset requested successfully", body = HttpResponseObjectEmpty),
        (status = 500, description = "Error requesting password reset", body = HttpResponseObjectEmptyError),
    ),
)]
pub async fn request_password_reset(
    body: web::Json<auth_model::PasswordResetRequestModel>,
    conf: web::Data<Arc<MyCondominiumConfig>>,
) -> HttpResponse {
    let conn = &mut establish_connection_pg(&conf);
    let email = body.email.trim().to_string();

    sleep(Duration::from_secs(5)).await;

    let user = UserModel::find_user_by_email(conn, email.clone()).ok();

    if user.is_some() {
        let pw_id = auth_model::PasswordResetModel::new_id(conn);

        let password_reset = auth_model::PasswordResetModel {
            id: pw_id,
            email: email.clone(),
            user_id: user.clone().unwrap().user.id,
            token: format!("{}{}", pw_id, Uuid::new_v4().to_string()),
            created_at: chrono::Utc::now().naive_utc(),
        };

        let rmq = RabbitMqClient::new(&conf.rabbitmq, "mycondominium_smtp".to_string())
            .await
            .unwrap();

        let first_name = match user.clone().unwrap().user.entity_type {
            UserTypes::Admin => user.unwrap().admin.unwrap().first_name,
            UserTypes::Resident => user.unwrap().resident.unwrap().first_name,
        };

        let parameters: Vec<SmtpTemplateData> = vec![
            SmtpTemplateData {
                key: "{{USER_NAME}}".to_string(),
                value: first_name,
            },
            SmtpTemplateData {
                key: "{{RESET_LINK}}".to_string(),
                value: format!(
                    "{}/redirect/onboarding/password_reset/{}",
                    conf.smtp.base_url, password_reset.token
                ),
            },
            SmtpTemplateData {
                key: "{{CURRENT_YEAR}}".to_string(),
                value: chrono::Utc::now().year().to_string(),
            },
        ];

        let template_data = crate::internal::smtp::smtp_templates::smtp_get_template(
            SmtpTemplate::PasswordReset,
            parameters,
        );

        let email = SmtpEmailPayload {
            to: email,
            subject: "Redefinição de Senha - MyCondominium".to_string(),
            body: template_data,
        };

        let payload = serde_json::to_vec(&email).unwrap();

        rmq.publish(&payload).await.unwrap();

        match password_reset.db_insert(conn) {
            Ok(_) => HttpResponse::Ok().json(HttpResponseObjectEmpty {
                error: false,
                message: "If an account with the provided email exists, it will be sent"
                    .to_string(),
            }),
            Err(e) => {
                log::error!("Error inserting password reset: {}", e);
                HttpResponse::InternalServerError().json(HttpResponseObjectEmptyError {
                    error: true,
                    message: "Error requesting password reset".to_string(),
                })
            }
        }
    } else {
        HttpResponse::Ok().json(HttpResponseObjectEmpty {
            error: false,
            message: "If an account with the provided email exists, it will be sent".to_string(),
        })
    }
}

#[utoipa::path(
    post,
    tag = "Authentication",
    path = "/password_reset",
    request_body = auth_model::PasswordResetChangeModel,
    responses (
        (status = 200, description = "Password Reset successfully", body = HttpResponseObjectEmpty),
        (status = 400, description = "Invalid token", body = HttpResponseObjectEmptyError),
        (status = 500, description = "Error resetting password", body = HttpResponseObjectEmptyError),
    ),
)]
pub async fn password_reset(
    body: web::Json<auth_model::PasswordResetChangeModel>,
    conf: web::Data<Arc<MyCondominiumConfig>>,
) -> HttpResponse {
    let conn = &mut establish_connection_pg(&conf);

    sleep(Duration::from_secs(3)).await;

    let pw_reset = auth_model::PasswordResetModel::table()
        .filter(crate::schema::password_reset::token.eq(body.clone().token))
        .first::<auth_model::PasswordResetModel>(conn)
        .ok();

    let body = body.clone();

    if pw_reset.is_some() {
        let hashed_password = match hash_password(body.password) {
            Ok(passwd) => passwd,
            Err(e) => {
                log::error!("Error hashing password: {}", e);
                return HttpResponse::InternalServerError().json(HttpResponseObjectEmptyError {
                    error: true,
                    message: "Error resetting password".to_string(),
                });
            }
        };

        let mut user = match UserModel::db_read_by_id(conn, pw_reset.clone().unwrap().user_id) {
            Ok(usr) => usr,
            Err(e) => {
                log::error!("Error reading user: {}", e);
                return HttpResponse::InternalServerError().json(HttpResponseObjectEmptyError {
                    error: true,
                    message: "Error resetting password".to_string(),
                });
            }
        };

        user.password = hashed_password;
        user.updated_at = chrono::Utc::now().naive_utc();

        match user.db_update(conn) {
            Ok(_) => {}
            Err(e) => {
                log::error!("Error updating user: {}", e);
                return HttpResponse::InternalServerError().json(HttpResponseObjectEmptyError {
                    error: true,
                    message: "Error resetting password".to_string(),
                });
            }
        };

        let rmq = RabbitMqClient::new(&conf.rabbitmq, "mycondominium_smtp".to_string())
            .await
            .unwrap();

        let (first_name, email) = match user.clone().entity_type {
            UserTypes::Admin => {
                match admin_model::AdminModel::db_read_by_id(conn, user.entity_id) {
                    Ok(adm) => (adm.first_name, adm.email),
                    Err(e) => {
                        log::error!("Error reading admin: {}", e);
                        return HttpResponse::InternalServerError().json(
                            HttpResponseObjectEmptyError {
                                error: true,
                                message: "Error getting user".to_string(),
                            },
                        );
                    }
                }
            }
            UserTypes::Resident => {
                match resident_model::ResidentModel::db_read_by_id(conn, user.entity_id) {
                    Ok(usr) => (usr.first_name, usr.email),
                    Err(e) => {
                        log::error!("Error reading resident: {}", e);
                        return HttpResponse::InternalServerError().json(
                            HttpResponseObjectEmptyError {
                                error: true,
                                message: "Error getting user".to_string(),
                            },
                        );
                    }
                }
            }
        };

        let parameters: Vec<SmtpTemplateData> = vec![
            SmtpTemplateData {
                key: "{{USER_NAME}}".to_string(),
                value: first_name,
            },
            SmtpTemplateData {
                key: "{{CURRENT_YEAR}}".to_string(),
                value: chrono::Utc::now().year().to_string(),
            },
        ];

        let template_data = crate::internal::smtp::smtp_templates::smtp_get_template(
            SmtpTemplate::PasswordResetWarning,
            parameters,
        );

        let email = SmtpEmailPayload {
            to: email,
            subject: "Senha Redefinida - MyCondominium".to_string(),
            body: template_data,
        };

        let payload = serde_json::to_vec(&email).unwrap();

        rmq.publish(&payload).await.unwrap();

        match auth_model::PasswordResetModel::db_delete_by_id(conn, pw_reset.unwrap().id) {
            Ok(_) => HttpResponse::Ok().json(HttpResponseObjectEmpty {
                error: false,
                message: "Your password has been changed successfully".to_string(),
            }),
            Err(e) => {
                log::error!("Error deleting password reset: {}", e);
                HttpResponse::InternalServerError().json(HttpResponseObjectEmptyError {
                    error: true,
                    message: "Error resetting password".to_string(),
                })
            }
        }
    } else {
        HttpResponse::BadRequest().json(HttpResponseObjectEmptyError {
            error: true,
            message: "Invalid token".to_string(),
        })
    }
}
