use super::*;
use crate::internal::rabbitmq::rabbitmq_client::RabbitMqClient;
use crate::internal::smtp::smtp_client::SmtpEmailPayload;
use crate::internal::smtp::smtp_templates::{SmtpTemplate, SmtpTemplateData};
use crate::models::resident_model::ResidentInviteModel;
use crate::utilities::user_utils::check_email_exist;
use base64::Engine;
use chrono::Datelike;
use std::time::Duration;
use tokio::time::sleep;

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

    let (role, _claims, _token) = match authenticate_user(req.clone(), conn, conf.clone()) {
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

    let community = match community_model::CommunityModel::db_read_by_id(conn, body.community_id) {
        Ok(comm) => comm,
        Err(_) => {
            return HttpResponse::BadRequest().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Invalid Community ID".to_string(),
            });
        }
    };

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

    let link_b64 = base64::engine::general_purpose::STANDARD.encode(new_obj.key.as_bytes());

    let parameters: Vec<SmtpTemplateData> = vec![
        SmtpTemplateData {
            key: "{{COMMUNITY_NAME}}".to_string(),
            value: community.name,
        },
        SmtpTemplateData {
            key: "{{INVITE_LINK}}".to_string(),
            value: format!("mycondominium://onboarding/register/{}", link_b64),
        },
        SmtpTemplateData {
            key: "{{CURRENT_YEAR}}".to_string(),
            value: chrono::Utc::now().year().to_string(),
        },
    ];

    let template_data = crate::internal::smtp::smtp_templates::smtp_get_template(
        SmtpTemplate::ResidentInvite,
        parameters,
    );

    let email = SmtpEmailPayload {
        to: body.email,
        subject: "Convite do MyCondominium".to_string(),
        body: template_data,
    };

    let payload = serde_json::to_vec(&email).unwrap();

    rmq.publish(&payload).await.unwrap();

    HttpResponse::Ok().json(HttpResponseObjectEmptyEntity {
        error: false,
        message: "Resident created successfully".to_string(),
        entity_id: Some(new_obj.id),
    })
}

#[utoipa::path(
    get,
    tag = "Resident",
    path = "/invite/list",
    params(
        ("page" = Option<i64>, Query, description = "Page number for pagination (default: 1)"),
        ("per_page" = Option<i64>, Query, description = "Number of items per page for pagination (default: 10)"),
    ),
    responses(
        (status = 200, description = "Got resident invites successfully", body = ResidentInviteListHttpResponse, headers(
            ("X-Total-Pages" = i64, description = "Total number of pages"),
            ("X-Remaining-Pages" = i64, description = "Remaining number of pages")
        )),
        (status = 401, description = "Unauthorized", body = HttpResponseObjectEmptyError),
        (status = 500, description = "Internal server error", body = HttpResponseObjectEmptyError)
    ),
    security(
        ("Token" = [])
    )
)]
pub async fn get_resident_invites(
    query: web::Query<PaginationParams>,
    req: HttpRequest,
    conf: web::Data<Arc<MyCondominiumConfig>>,
) -> HttpResponse {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(10);
    let offset = (page - 1) * per_page;

    let conn = &mut establish_connection_pg(&conf);

    let (role, _claims, _token) = match authenticate_user(req.clone(), conn, conf) {
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

    let total_items = match resident_model::ResidentInviteModel::db_count_all_matching_community(
        role.clone(),
        conn,
    ) {
        Ok(count) => count,
        Err(_) => {
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Error counting Invites".to_string(),
            });
        }
    };

    match resident_model::ResidentInviteModel::db_read_all_matching_community_by_range(
        role, conn, per_page, offset,
    ) {
        Ok(res) => {
            let total_pages = (total_items as f64 / per_page as f64).ceil() as i64;
            let remaining_pages = total_pages - page;

            HttpResponse::Ok()
                .insert_header((
                    header::HeaderName::from_static("x-total-pages"),
                    total_pages.to_string(),
                ))
                .insert_header((
                    header::HeaderName::from_static("x-remaining-pages"),
                    remaining_pages.to_string(),
                ))
                .json(HttpResponseObject {
                    error: false,
                    message: "Got resident invites successfully".to_string(),
                    object: Some(res),
                })
        }
        Err(e) => {
            log::error!("Error getting resident invites: {}", e);
            HttpResponse::InternalServerError().json(HttpResponseObjectEmpty {
                error: true,
                message: format!("Error getting resident invites: {}", e),
            })
        }
    }
}

#[utoipa::path(
    get,
    tag = "Resident",
    path = "/invite/count",
    responses(
        (status = 200, description = "Got resident invites successfully", body = HttpResponseObject<i64>),
        (status = 401, description = "Unauthorized", body = HttpResponseObjectEmptyError),
        (status = 500, description = "Internal server error", body = HttpResponseObjectEmptyError)
    ),
    security(
        ("Token" = [])
    )
)]
pub async fn count_resident_invite(
    req: HttpRequest,
    conf: web::Data<Arc<MyCondominiumConfig>>,
) -> HttpResponse {
    let conn = &mut establish_connection_pg(&conf);

    let (role, _claims, _token) = match authenticate_user(req.clone(), conn, conf) {
        Ok((role, claims, token)) => match role.role {
            UserRoles::Root => (role, claims, token),
            UserRoles::Admin => (role, claims, token),
            _ => {
                return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
                    error: true,
                    message: "Unauthorized".to_string(),
                });
            }
        },
        Err(_) => {
            return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Unauthorized".to_string(),
            });
        }
    };

    match resident_model::ResidentInviteModel::db_count_all_matching_community(role, conn) {
        Ok(res) => HttpResponse::Ok().json(HttpResponseObject {
            error: false,
            message: "Got Resident invites successfully".to_string(),
            object: Some(res),
        }),
        Err(e) => {
            log::error!("Error counting Resident invites: {}", e);
            HttpResponse::InternalServerError().json(HttpResponseObjectEmpty {
                error: true,
                message: format!("Error counting Resident invites: {}", e),
            })
        }
    }
}

#[utoipa::path(
    get,
    tag = "Resident",
    path = "/invite/get/{id}",
    params(
        ("id" = Uuid, Path, description = "Resident Invite ID"),
    ),
    responses(
        (status = 200, description = "Got resident invite successfully", body = ResidentInviteGetHttpResponse),
        (status = 400, description = "Invalid Resident Invite ID format or Resident Invite ID is required", body = HttpResponseObjectEmptyError),
        (status = 401, description = "Unauthorized", body = HttpResponseObjectEmptyError),
        (status = 500, description = "Internal server error", body = HttpResponseObjectEmptyError)
    ),
    security(
        ("Token" = [])
    )
)]
pub async fn get_resident_invite_by_id(
    id: web::Path<String>,
    req: HttpRequest,
    conf: web::Data<Arc<MyCondominiumConfig>>,
) -> HttpResponse {
    let id = id.into_inner();

    let conn = &mut establish_connection_pg(&conf);

    let id = match Uuid::parse_str(&id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return HttpResponse::BadRequest().json(HttpResponseObjectEmpty {
                error: true,
                message: "Invalid Resident Invite ID format".to_string(),
            });
        }
    };

    let role = match authenticate_user(req.clone(), conn, conf) {
        Ok((role, _claims, _token)) => {
            if role.role == UserRoles::Root || role.role == UserRoles::Admin {
                role
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

    match resident_model::ResidentInviteModel::db_read_by_id_matching_community(role, conn, id) {
        Ok(user_req) => {
            let invite = ResidentInviteModel {
                id: user_req.id,
                email: user_req.email,
                community_id: user_req.community_id,
                key: "".to_string(),
                created_at: user_req.created_at,
            };
            HttpResponse::Ok().json(HttpResponseObject {
                error: false,
                message: "Got resident invite successfully".to_string(),
                object: Some(invite),
            })
        }
        Err(e) => {
            log::error!("Error getting resident invite: {}", e);
            HttpResponse::InternalServerError().json(HttpResponseObjectEmpty {
                error: true,
                message: format!("Error getting resident invite: {}", e),
            })
        }
    }
}

#[utoipa::path(
    delete,
    tag = "Resident",
    path = "/invite/delete/{id}",
    params(
        ("id" = Uuid, Path, description = "Resident Invite ID"),
    ),
    responses (
        (status = 200, description = "Resident Invite deleted successfully", body = HttpResponseObjectEmpty),
        (status = 400, description = "Invalid Resident Invite ID format", body = HttpResponseObjectEmptyError),
        (status = 401, description = "Unauthorized", body = HttpResponseObjectEmptyError),
        (status = 500, description = "Error deleting Resident", body = HttpResponseObjectEmptyError),
    ),
    security(
        ("Token" = [])
    )
)]
pub async fn delete_resident_invite(
    id: web::Path<String>,
    req: HttpRequest,
    conf: web::Data<Arc<MyCondominiumConfig>>,
) -> HttpResponse {
    let conn = &mut establish_connection_pg(&conf);

    let id = match Uuid::parse_str(&id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return HttpResponse::BadRequest().json(HttpResponseObjectEmpty {
                error: true,
                message: "Invalid Resident ID format".to_string(),
            });
        }
    };

    let invite = match resident_model::ResidentInviteModel::db_read_by_id(conn, id) {
        Ok(res) => res,
        Err(_) => {
            return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Unauthorized".to_string(),
            });
        }
    };

    let (role, _claims, _token) = match authenticate_user(req.clone(), conn, conf) {
        Ok((role, claims, token)) => (role, claims, token),
        Err(_) => {
            return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Unauthorized".to_string(),
            });
        }
    };

    match role.role {
        UserRoles::Root => {}
        UserRoles::Admin => {
            if role.community_id.is_none() {
                return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
                    error: true,
                    message: "Unauthorized".to_string(),
                });
            }

            if role.community_id.unwrap() != invite.community_id {
                return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
                    error: true,
                    message: "Unauthorized".to_string(),
                });
            }
        }
        _ => {
            return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Unauthorized".to_string(),
            });
        }
    }

    match resident_model::ResidentInviteModel::db_delete_by_id(conn, id) {
        Ok(_) => HttpResponse::Ok().json(HttpResponseObjectEmpty {
            error: false,
            message: "Resident Invite deleted successfully".to_string(),
        }),
        Err(e) => {
            log::error!("Error deleting resident invite: {}", e);
            HttpResponse::InternalServerError().json(HttpResponseObjectEmptyError {
                error: true,
                message: format!("Error deleting resident invite: {}", e),
            })
        }
    }
}

#[utoipa::path(
    post,
    tag = "Resident",
    path = "/invite/signup",
    request_body = resident_model::ResidentModelNewInvite,
    responses (
        (status = 200, description = "Resident added successfully", body = HttpResponseObjectEmptyEntity),
        (status = 400, description = "Error adding resident", body = HttpResponseObjectEmptyError),
        (status = 401, description = "Unauthorized", body = HttpResponseObjectEmptyError),
        (status = 500, description = "Error adding resident", body = HttpResponseObjectEmptyError),
    )
)]
pub async fn new_resident_by_invite(
    body: web::Json<resident_model::ResidentModelNewInvite>,
    conf: web::Data<Arc<MyCondominiumConfig>>,
) -> HttpResponse {
    let conn = &mut establish_connection_pg(&conf);

    let body = body.into_inner();

    if let Err(validation_errors) = body.validate() {
        return HttpResponse::BadRequest().json(validation_errors);
    }

    sleep(Duration::from_secs(5)).await;

    let invite = match resident_model::ResidentInviteModel::table()
        .filter(resident_invites::key.eq(&body.key))
        .first::<resident_model::ResidentInviteModel>(conn)
    {
        Ok(inv) => inv,
        Err(_) => {
            return HttpResponse::BadRequest().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Invalid Invite Key".to_string(),
            });
        }
    };

    let new_obj = resident_model::ResidentModel {
        id: resident_model::ResidentModel::new_id_user(conn),
        first_name: body.first_name,
        last_name: body.last_name,
        unit_number: body.unit_number,
        address: body.address,
        phone: body.phone,
        email: invite.email,
        date_of_birth: body.date_of_birth,
        is_active: true,
        created_at: chrono::Utc::now().naive_utc(),
        updated_at: chrono::Utc::now().naive_utc(),
    };

    match new_obj.db_insert(conn) {
        Ok(_) => (),
        Err(e) => {
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmptyError {
                error: true,
                message: format!("Error creating resident: {}", e),
            });
        }
    };

    let hashed_password = match hash_password(body.password) {
        Ok(passwd) => passwd,
        Err(e) => {
            log::error!("Error creating resident: {}", e);
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Error creating resident".to_string(),
            });
        }
    };

    let new_obj_user = user_model::UserModel {
        id: user_model::UserModel::new_id(conn),
        entity_id: new_obj.id,
        entity_type: UserTypes::Resident,
        admin_id: None,
        resident_id: Some(new_obj.id),
        password: hashed_password,
        created_at: chrono::Utc::now().naive_utc(),
        updated_at: chrono::Utc::now().naive_utc(),
    };

    match new_obj_user.db_insert(conn) {
        Ok(_) => (),
        Err(e) => {
            log::error!("Error creating resident: {}", e);
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Error creating resident".to_string(),
            });
        }
    };

    let new_obj_user_role = user_role_model::UserRoleModel {
        id: user_role_model::UserRoleModel::new_id(conn),
        user_id: new_obj_user.id,
        role: UserRoles::Resident,
        community_id: Some(invite.community_id),
        created_at: chrono::Utc::now().naive_utc(),
        updated_at: chrono::Utc::now().naive_utc(),
    };

    match resident_model::ResidentInviteModel::db_delete_by_id(conn, invite.id) {
        Ok(_) => (),
        Err(e) => {
            log::error!("Error deleting resident invite: {}", e);
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Error creating resident".to_string(),
            });
        }
    }

    match new_obj_user_role.db_insert(conn) {
        Ok(_) => (),
        Err(e) => {
            log::error!("Error creating resident: {}", e);
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Error creating resident".to_string(),
            });
        }
    };

    HttpResponse::Ok().json(HttpResponseObjectEmptyEntity {
        error: false,
        message: "Resident created successfully".to_string(),
        entity_id: Some(new_obj.id),
    })
}
