use super::*;
use crate::internal::rabbitmq::rabbitmq_client::RabbitMqClient;
use crate::internal::smtp::smtp_client::SmtpEmailPayload;
use crate::utilities::user_utils::{check_email_exist, user_check_email_valid};

#[utoipa::path(
    post,
    tag = "Resident",
    path = "/new",
    request_body = resident_model::ResidentModelNew,
    responses (
        (status = 200, description = "Resident added successfully", body = HttpResponseObjectEmptyEntity),
        (status = 400, description = "Error adding resident", body = HttpResponseObjectEmptyError),
        (status = 401, description = "Unauthorized", body = HttpResponseObjectEmptyError),
        (status = 500, description = "Error adding resident", body = HttpResponseObjectEmptyError),
    ),
    security(
        ("Token" = [])
    )
)]
pub async fn new_resident(
    body: web::Json<resident_model::ResidentModelNew>,
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
        || (role.role == UserRoles::Admin && role.community_id == body.community_id))
    {
        return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
            error: true,
            message: "Unauthorized".to_string(),
        });
    }

    if let Err(validation_errors) = body.validate() {
        return HttpResponse::BadRequest().json(validation_errors);
    }

    match check_email_exist(conn, body.email.clone()) {
        Ok(()) => (),
        Err(e) => {
            log::error!("Error creating resident: {}", e);
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Error creating resident: Email already in use".to_string(),
            });
        }
    }

    let new_obj = resident_model::ResidentModel {
        id: Uuid::new_v4(),
        first_name: body.first_name,
        last_name: body.last_name,
        unit_number: body.unit_number,
        address: body.address,
        phone: body.phone,
        email: body.email,
        date_of_birth: body.date_of_birth,
        is_active: body.is_active,
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

    match body.password {
        Some(password) => {
            let hashed_password = match hash_password(password) {
                Ok(passwd) => passwd,
                Err(e) => {
                    log::error!("Error creating resident: {}", e);
                    return HttpResponse::InternalServerError().json(
                        HttpResponseObjectEmptyError {
                            error: true,
                            message: "Error creating resident".to_string(),
                        },
                    );
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
                    return HttpResponse::InternalServerError().json(
                        HttpResponseObjectEmptyError {
                            error: true,
                            message: "Error creating resident".to_string(),
                        },
                    );
                }
            };

            let new_obj_user_role = user_role_model::UserRoleModel {
                id: user_role_model::UserRoleModel::new_id(conn),
                user_id: new_obj_user.id,
                role: UserRoles::Resident,
                community_id: body.community_id,
                created_at: chrono::Utc::now().naive_utc(),
                updated_at: chrono::Utc::now().naive_utc(),
            };

            match new_obj_user_role.db_insert(conn) {
                Ok(_) => (),
                Err(e) => {
                    log::error!("Error creating resident: {}", e);
                    return HttpResponse::InternalServerError().json(
                        HttpResponseObjectEmptyError {
                            error: true,
                            message: "Error creating resident".to_string(),
                        },
                    );
                }
            };

            /*
            let rmq = RabbitMqClient::new(&conf.rabbitmq, "mycondominium_smtp".to_string()).await.unwrap();
            let email = SmtpEmailPayload {
                to: "example@al3xdev.com".to_string(),
                subject: "Test - New Resident".to_string(),
                body: "A New Resident has been added.".to_string(),
            };

            let payload = serde_json::to_vec(&email).unwrap();

            rmq.publish(&payload).await.unwrap();

             */

            HttpResponse::Ok().json(HttpResponseObjectEmptyEntity {
                error: false,
                message: "Resident created successfully".to_string(),
                entity_id: Some(new_obj.id),
            })
        }
        None => {
            todo!("Implement sending email to new resident to create password");
        }
    }
}

#[utoipa::path(
    put,
    tag = "Resident",
    path = "/update/{id}",
    request_body = resident_model::ResidentModelNew,
    params(
        ("id" = Uuid, Path, description = "Resident ID"),
    ),
    responses (
        (status = 200, description = "Resident updated successfully", body = HttpResponseObjectEmpty),
        (status = 400, description = "Error updating Resident", body = HttpResponseObjectEmptyError),
        (status = 401, description = "Unauthorized", body = HttpResponseObjectEmptyError),
        (status = 500, description = "Internal server error", body = HttpResponseObjectEmptyError),
    ),
    security(
        ("Token" = [])
    )
)]
pub async fn update_resident(
    id: web::Path<String>,
    body: web::Json<resident_model::ResidentModelNew>,
    req: HttpRequest,
    conf: web::Data<Arc<MyCondominiumConfig>>,
) -> HttpResponse {
    let conn = &mut establish_connection_pg(&conf);
    let body = body.into_inner();

    match authenticate_user(req.clone(), conn, conf) {
        Ok((role, claims, token)) => {
            if role.role == UserRoles::Admin {
                if role.role != UserRoles::Root {
                    if body.community_id.is_none() || role.community_id.is_none() {
                        return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
                            error: true,
                            message: "Unauthorized".to_string(),
                        });
                    }

                    if !(role.role == UserRoles::Admin
                        && body.community_id.unwrap() == role.community_id.unwrap())
                    {
                        return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
                            error: true,
                            message: "Unauthorized".to_string(),
                        });
                    }
                }
            } else if role.role == UserRoles::Root && role.role != UserRoles::Root {
                return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
                    error: true,
                    message: "Unauthorized".to_string(),
                });
            } else {
                return HttpResponse::BadRequest().json(HttpResponseObjectEmptyError {
                    error: true,
                    message: "Invalid Admin Role".to_string(),
                });
            }
        }
        Err(_) => {
            return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Unauthorized".to_string(),
            });
        }
    }

    if let Err(validation_errors) = body.validate() {
        return HttpResponse::BadRequest().json(validation_errors);
    }

    let id = match Uuid::parse_str(&id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return HttpResponse::BadRequest().json(HttpResponseObjectEmpty {
                error: true,
                message: "Invalid Resident ID format".to_string(),
            });
        }
    };

    let curr_obj = match resident_model::ResidentModel::db_read_by_id(conn, id) {
        Ok(user_req) => user_req,
        Err(e) => {
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmpty {
                error: true,
                message: format!("Error getting resident: {}", e),
            });
        }
    };

    match user_check_email_valid(conn, body.email.clone(), curr_obj.email) {
        Ok(()) => (),
        Err(e) => {
            return HttpResponse::BadRequest().json(HttpResponseObjectEmpty {
                error: true,
                message: "Email already in use".to_string(),
            });
        }
    }

    let new_obj = resident_model::ResidentModel {
        id: curr_obj.id,
        first_name: body.first_name,
        last_name: body.last_name,
        unit_number: body.unit_number,
        address: body.address,
        phone: body.phone,
        email: body.email,
        date_of_birth: body.date_of_birth,
        is_active: body.is_active,
        created_at: curr_obj.created_at,
        updated_at: chrono::Utc::now().naive_utc(),
    };

    match new_obj.db_update(conn) {
        Ok(_) => HttpResponse::Ok().json(HttpResponseObjectEmpty {
            error: false,
            message: "Resident updated successfully".to_string(),
        }),
        Err(e) => HttpResponse::Ok().json(HttpResponseObjectEmpty {
            error: true,
            message: format!("Error creating resident: {}", e),
        }),
    }
}

#[utoipa::path(
    delete,
    tag = "Resident",
    path = "/delete/{id}",
    params(
        ("id" = Uuid, Path, description = "Resident ID"),
    ),
    responses (
        (status = 200, description = "Resident deleted successfully", body = HttpResponseObjectEmpty),
        (status = 400, description = "Invalid Resident ID format", body = HttpResponseObjectEmptyError),
        (status = 401, description = "Unauthorized", body = HttpResponseObjectEmptyError),
        (status = 500, description = "Error deleting Resident", body = HttpResponseObjectEmptyError),
    ),
    security(
        ("Token" = [])
    )
)]
pub async fn delete_resident(
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

    let resident = match resident_model::ResidentModel::db_read_by_id(conn, id) {
        Ok(res) => res,
        Err(e) => {
            return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Unauthorized".to_string(),
            });
        }
    };

    let res_user = match user_model::UserModel::table()
        .filter(crate::schema::users::resident_id.eq(resident.id))
        .first::<user_model::UserModel>(conn)
    {
        Ok(res_user) => res_user,
        Err(e) => {
            return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Unauthorized".to_string(),
            });
        }
    };

    let res_user_role = match user_role_model::UserRoleModel::table()
        .filter(crate::schema::user_roles::user_id.eq(res_user.id))
        .first::<user_role_model::UserRoleModel>(conn)
    {
        Ok(adm_user_role) => adm_user_role,
        Err(e) => {
            return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Unauthorized".to_string(),
            });
        }
    };

    match authenticate_user(req.clone(), conn, conf) {
        Ok((role, claims, token)) => {
            if role.role != UserRoles::Root {
                if res_user_role.community_id.is_none() || role.community_id.is_none() {
                    return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
                        error: true,
                        message: "Unauthorized".to_string(),
                    });
                }

                if !(role.role == UserRoles::Admin
                    && res_user_role.community_id.unwrap() == role.community_id.unwrap())
                {
                    return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
                        error: true,
                        message: "Unauthorized".to_string(),
                    });
                }
            }
        }
        Err(_) => {
            return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Unauthorized".to_string(),
            });
        }
    }

    match resident_model::ResidentModel::db_delete_by_id(conn, id) {
        Ok(_) => HttpResponse::Ok().json(HttpResponseObjectEmpty {
            error: false,
            message: "Resident deleted successfully".to_string(),
        }),
        Err(e) => HttpResponse::InternalServerError().json(HttpResponseObjectEmptyError {
            error: true,
            message: format!("Error deleting resident: {}", e),
        }),
    }
}
