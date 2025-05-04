use super::*;
use crate::internal::rabbitmq::rabbitmq_client::RabbitMqClient;
use crate::internal::smtp::smtp_client::SmtpEmailPayload;
use crate::utilities::user_utils::check_email_exist;

#[utoipa::path(
    post,
    tag = "Resident",
    path = "/invite/new",
    request_body = resident_model::ResidentInviteModelNew,
    responses (
        (status = 200, description = "New Resident invited successfully", body = HttpResponseObjectEmpty),
        (status = 401, description = "Unauthorized", body = HttpResponseObjectEmptyError),
        (status = 500, description = "Error inviting Resident", body = HttpResponseObjectEmptyError),
    ),
    security(
        ("Token" = [])
    )
)]
pub async fn new_resident_invite(
    body: web::Json<resident_model::ResidentInviteModelNew>,
    req: HttpRequest,
    conf: web::Data<Arc<MyCondominiumConfig>>,
) -> HttpResponse {
    let conn = &mut establish_connection_pg(&conf);

    let body = body.into_inner();

    let (role, claims, token) = match authenticate_user(req.clone(), conn, conf.clone()) {
        Ok((role, claims, token)) => {
            if role.role == UserRoles::Root || role.role == UserRoles::Admin {
                (role, claims, token)
            } else {
                return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
                    error: true,
                    message: "Unauthorized".to_string(),
                });
            }
        }
        Err(_) => {
            return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Unauthorized".to_string(),
            });
        }
    };

    if !(role.role == UserRoles::Root
        || (role.role == UserRoles::Admin && role.community_id == Some(body.community_id)))
    {
        return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
            error: true,
            message: "Unauthorized".to_string(),
        });
    }

    if let Err(validation_errors) = body.validate() {
        return HttpResponse::BadRequest().json(validation_errors);
    }

    if community_model::CommunityModel::db_read_by_id(conn, body.community_id).is_err() {
        return HttpResponse::BadRequest().json(HttpResponseObjectEmptyError {
            error: true,
            message: "Invalid Community ID".to_string(),
        });
    }

    match check_email_exist(conn, body.email.clone()) {
        Ok(()) => (),
        Err(e) => {
            log::error!("Error inviting resident: {}", e);
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Error inviting resident: Email already in use".to_string(),
            });
        }
    }

    let new_id = resident_model::ResidentInviteModel::new_id(conn);

    let new_obj = resident_model::ResidentInviteModel {
        id: new_id,
        email: body.email.clone(),
        community_id: body.community_id,
        key: format!(
            "{}-{}",
            new_id.to_string(),
            uuid::Uuid::new_v4().to_string()
        ),
        created_at: chrono::Utc::now().naive_utc(),
    };

    match new_obj.db_insert(conn) {
        Ok(_) => (),
        Err(e) => {
            log::error!("Internal Server Error creating resident invitation: {}", e);
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmptyError {
                error: true,
                message: format!("Internal Server Error creating resident invitation: {}", e),
            });
        }
    };

    let rmq = RabbitMqClient::new(&conf.rabbitmq, "mycondominium_smtp".to_string())
        .await
        .unwrap();
    let email = SmtpEmailPayload {
        to: body.email,
        subject: "MyCondominium Invitation".to_string(),
        body: format!(
            "Create your account at {}/signup/invitation?id={}",
            conf.smtp.base_url, new_obj.key
        ),
    };

    let payload = serde_json::to_vec(&email).unwrap();

    rmq.publish(&payload).await.unwrap();

    HttpResponse::Ok().json(HttpResponseObjectEmptyEntity {
        error: false,
        message: "Resident created successfully".to_string(),
        entity_id: Some(new_obj.id),
    })
}
