use super::*;
use crate::internal::rabbitmq::rabbitmq_client::RabbitMqClient;
use crate::internal::smtp::smtp_client::SmtpEmailPayload;
use crate::internal::smtp::smtp_templates::{SmtpTemplate, SmtpTemplateData};
use crate::utilities::user_utils::check_email_exist;
use chrono::Datelike;
use std::time::Duration;
use tokio::time::sleep;

#[utoipa::path(
    post,
    tag = "Authentication",
    path = "/new_admin_self_service",
    request_body = auth_model::AuthAdminNewSelfServiceModel,
    responses (
        (status = 200, description = "Admin added successfully", body = HttpResponseObjectEmptyEntity),
        (status = 400, description = "Error adding admin", body = HttpResponseObjectEmptyError),
        (status = 401, description = "Unauthorized", body = HttpResponseObjectEmptyError),
        (status = 500, description = "Error adding admin", body = HttpResponseObjectEmptyError),
    ),
)]
pub async fn new_admin_self_service(
    body: web::Json<auth_model::AuthAdminNewSelfServiceModel>,
    req: HttpRequest,
    conf: web::Data<Arc<MyCondominiumConfig>>,
) -> HttpResponse {
    sleep(Duration::from_secs(10)).await;
    let conn = &mut establish_connection_pg(&conf);

    let body = body.into_inner();

    if let Err(validation_errors) = body.validate() {
        return HttpResponse::BadRequest().json(validation_errors);
    }

    match check_email_exist(conn, body.admin.email.clone()) {
        Ok(_) => (),
        Err(e) => {
            log::error!("Error creating admin: {}", e);
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Error creating admin: Email already in use".to_string(),
            });
        }
    }

    let new_obj_community = community_model::CommunityModel {
        id: community_model::CommunityModel::new_id(conn),
        name: body.community.name,
        short_name: body.community.short_name,
        address: body.community.address,
        created_at: chrono::Utc::now().naive_utc(),
        updated_at: chrono::Utc::now().naive_utc(),
    };

    match new_obj_community.db_insert(conn) {
        Ok(_) => (),
        Err(e) => {
            log::error!("Error creating Community: {}", e);
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Error creating Community".to_string(),
            });
        }
    };

    let new_obj_admin = admin_model::AdminModel {
        id: admin_model::AdminModel::new_id_user(conn),
        first_name: body.admin.first_name,
        last_name: body.admin.last_name,
        phone: body.admin.phone,
        email: body.admin.email,
        created_at: chrono::Utc::now().naive_utc(),
        updated_at: chrono::Utc::now().naive_utc(),
    };

    match new_obj_admin.db_insert(conn) {
        Ok(_) => (),
        Err(e) => {
            log::error!("Error creating admin: {}", e);
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Error creating admin".to_string(),
            });
        }
    };

    let hashed_password = match hash_password(body.admin.password) {
        Ok(passwd) => passwd,
        Err(e) => {
            log::error!("Error creating admin: {}", e);
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Error creating admin".to_string(),
            });
        }
    };

    let new_obj_user = user_model::UserModel {
        id: user_model::UserModel::new_id(conn),
        entity_id: new_obj_admin.id,
        entity_type: UserTypes::Admin,
        admin_id: Some(new_obj_admin.id),
        resident_id: None,
        password: hashed_password,
        created_at: chrono::Utc::now().naive_utc(),
        updated_at: chrono::Utc::now().naive_utc(),
    };

    match new_obj_user.db_insert(conn) {
        Ok(_) => (),
        Err(e) => {
            log::error!("Error creating admin: {}", e);
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Error creating admin".to_string(),
            });
        }
    };

    let new_obj_user_role = user_role_model::UserRoleModel {
        id: user_role_model::UserRoleModel::new_id(conn),
        user_id: new_obj_user.id,
        role: UserRoles::Admin,
        community_id: Some(new_obj_community.id),
        created_at: chrono::Utc::now().naive_utc(),
        updated_at: chrono::Utc::now().naive_utc(),
    };

    match new_obj_user_role.db_insert(conn) {
        Ok(_) => (),
        Err(e) => {
            log::error!("Error creating admin: {}", e);
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Error creating admin".to_string(),
            });
        }
    };

    let rmq = RabbitMqClient::new(&conf.rabbitmq, "mycondominium_smtp".to_string())
        .await
        .unwrap();

    let url = conf.smtp.base_url.clone();

    let parameters: Vec<SmtpTemplateData> = vec![
        SmtpTemplateData {
            key: "{{USER_NAME}}".to_string(),
            value: new_obj_admin.first_name,
        },
        SmtpTemplateData {
            key: "{{APP_LINK}}".to_string(),
            value: url,
        },
        SmtpTemplateData {
            key: "{{CURRENT_YEAR}}".to_string(),
            value: chrono::Utc::now().year().to_string(),
        },
    ];

    let template_data = crate::internal::smtp::smtp_templates::smtp_get_template(
        SmtpTemplate::NewAccount,
        parameters,
    );

    let email = SmtpEmailPayload {
        to: new_obj_admin.email,
        subject: "Bem-Vindo ao MyCondominium".to_string(),
        body: template_data,
    };

    let payload = serde_json::to_vec(&email).unwrap();

    rmq.publish(&payload).await.unwrap();

    HttpResponse::Ok().json(HttpResponseObjectEmptyEntity {
        error: false,
        message: "Admin created successfully".to_string(),
        entity_id: Some(new_obj_admin.id),
    })
}
