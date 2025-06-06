use super::*;
use crate::internal::roles::UserRoles;
use crate::utilities::auth_utils::hash_password;
use crate::utilities::user_utils::{check_email_exist, user_check_email_valid};

#[utoipa::path(
    post,
    tag = "Admin",
    path = "/new",
    request_body = admin_model::AdminModelNew,
    responses (
        (status = 200, description = "Admin added successfully", body = HttpResponseObjectEmptyEntity),
        (status = 400, description = "Error adding admin", body = HttpResponseObjectEmptyError),
        (status = 401, description = "Unauthorized", body = HttpResponseObjectEmptyError),
        (status = 500, description = "Error adding admin", body = HttpResponseObjectEmptyError),
    ),
    security(
        ("Token" = [])
    )
)]
pub async fn new_admin(
    body: web::Json<admin_model::AdminModelNew>,
    req: HttpRequest,
    conf: web::Data<Arc<MyCondominiumConfig>>,
) -> HttpResponse {
    let conn = &mut establish_connection_pg(&conf);

    let body = body.into_inner();

    if body.role == UserRoles::Root {
        let total_root_admins = match user_role_model::UserRoleModel::count_root_admins(conn) {
            Ok(num) => num,
            Err(e) => {
                log::error!("Error creating admin: {}", e);
                return HttpResponse::InternalServerError().json(HttpResponseObjectEmptyError {
                    error: true,
                    message: "Error creating admin".to_string(),
                });
            }
        };
        if total_root_admins > 0 {
            match authenticate_user(req.clone(), conn, conf) {
                Ok((role, _claims, _token)) => {
                    if role.role != UserRoles::Root {
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
            }
        }
    } else if body.role == UserRoles::Admin {
        match authenticate_user(req.clone(), conn, conf) {
            Ok((role, _claims, _token)) => {
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
            }
            Err(_) => {
                return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
                    error: true,
                    message: "Unauthorized".to_string(),
                });
            }
        }
    } else {
        return HttpResponse::BadRequest().json(HttpResponseObjectEmptyError {
            error: true,
            message: "Invalid Admin Role".to_string(),
        });
    }

    if let Err(validation_errors) = body.validate() {
        return HttpResponse::BadRequest().json(validation_errors);
    }

    match check_email_exist(conn, body.email.clone()) {
        Ok(()) => (),
        Err(e) => {
            log::error!("Error creating admin: {}", e);
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Error creating admin: Email already in use".to_string(),
            });
        }
    }

    let new_obj = admin_model::AdminModel {
        id: admin_model::AdminModel::new_id_user(conn),
        first_name: body.first_name,
        last_name: body.last_name,
        phone: body.phone,
        email: body.email,
        created_at: chrono::Utc::now().naive_utc(),
        updated_at: chrono::Utc::now().naive_utc(),
    };

    match new_obj.db_insert(conn) {
        Ok(_) => (),
        Err(e) => {
            log::error!("Error creating admin: {}", e);
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Error creating admin".to_string(),
            });
        }
    };

    let hashed_password = match hash_password(body.password) {
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
        entity_id: new_obj.id,
        entity_type: UserTypes::Admin,
        admin_id: Some(new_obj.id),
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
        role: body.role,
        community_id: body.community_id,
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

    HttpResponse::Ok().json(HttpResponseObjectEmptyEntity {
        error: false,
        message: "Admin created successfully".to_string(),
        entity_id: Some(new_obj.id),
    })
}

#[utoipa::path(
    put,
    tag = "Admin",
    path = "/update/{id}",
    request_body = admin_model::AdminModelNew,
    params(
        ("id" = Uuid, Path, description = "Admin ID"),
    ),
    responses (
        (status = 200, description = "Admin updated successfully", body = HttpResponseObjectEmpty),
        (status = 400, description = "Error updating Admin", body = HttpResponseObjectEmptyError),
        (status = 401, description = "Unauthorized", body = HttpResponseObjectEmptyError),
        (status = 500, description = "Internal server error", body = HttpResponseObjectEmptyError),
    ),
    security(
        ("Token" = [])
    )
)]
pub async fn update_admin(
    id: web::Path<String>,
    body: web::Json<admin_model::AdminModelNew>,
    req: HttpRequest,
    conf: web::Data<Arc<MyCondominiumConfig>>,
) -> HttpResponse {
    let conn = &mut establish_connection_pg(&conf);
    let body = body.into_inner();

    match authenticate_user(req.clone(), conn, conf) {
        Ok((role, _claims, _token)) => {
            if body.role == UserRoles::Admin {
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
            } else if body.role == UserRoles::Root && role.role != UserRoles::Root {
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
                message: "Invalid Admin ID format".to_string(),
            });
        }
    };

    let curr_obj = match admin_model::AdminModel::db_read_by_id(conn, id) {
        Ok(ent_req) => ent_req,
        Err(e) => {
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmpty {
                error: true,
                message: format!("Error getting admin: {}", e),
            });
        }
    };

    match user_check_email_valid(conn, body.email.clone(), curr_obj.email) {
        Ok(()) => (),
        Err(_) => {
            return HttpResponse::BadRequest().json(HttpResponseObjectEmpty {
                error: true,
                message: "Email already in use".to_string(),
            });
        }
    }

    let new_obj = admin_model::AdminModel {
        id: curr_obj.id,
        first_name: body.first_name,
        last_name: body.last_name,
        phone: body.phone,
        email: body.email,
        created_at: curr_obj.created_at,
        updated_at: chrono::Utc::now().naive_utc(),
    };

    match new_obj.db_update(conn) {
        Ok(_) => HttpResponse::Ok().json(HttpResponseObjectEmpty {
            error: false,
            message: "Admin updated successfully".to_string(),
        }),
        Err(e) => HttpResponse::Ok().json(HttpResponseObjectEmpty {
            error: true,
            message: format!("Error creating admin: {}", e),
        }),
    }
}

#[utoipa::path(
    delete,
    tag = "Admin",
    path = "/delete/{id}",
    params(
        ("id" = Uuid, Path, description = "Admin ID"),
    ),
    responses (
        (status = 200, description = "Admin deleted successfully", body = HttpResponseObjectEmpty),
        (status = 400, description = "Invalid Admin ID format", body = HttpResponseObjectEmptyError),
        (status = 401, description = "Unauthorized", body = HttpResponseObjectEmptyError),
        (status = 500, description = "Error deleting Admin", body = HttpResponseObjectEmptyError),
    ),
    security(
        ("Token" = [])
    )
)]
pub async fn delete_admin(
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
                message: "Invalid Admin ID format".to_string(),
            });
        }
    };

    let adm = match admin_model::AdminModel::db_read_by_id(conn, id) {
        Ok(adm) => adm,
        Err(_) => {
            return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Unauthorized".to_string(),
            });
        }
    };

    let adm_user = match user_model::UserModel::table()
        .filter(crate::schema::users::admin_id.eq(adm.id))
        .first::<user_model::UserModel>(conn)
    {
        Ok(adm_user) => adm_user,
        Err(_) => {
            return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Unauthorized".to_string(),
            });
        }
    };

    let adm_user_role = match user_role_model::UserRoleModel::table()
        .filter(crate::schema::user_roles::user_id.eq(adm_user.id))
        .first::<user_role_model::UserRoleModel>(conn)
    {
        Ok(adm_user_role) => adm_user_role,
        Err(_) => {
            return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Unauthorized".to_string(),
            });
        }
    };

    match authenticate_user(req.clone(), conn, conf) {
        Ok((role, _claims, _token)) => {
            if role.role != UserRoles::Root {
                if adm_user_role.community_id.is_none() || role.community_id.is_none() {
                    return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
                        error: true,
                        message: "Unauthorized".to_string(),
                    });
                }

                if !(role.role == UserRoles::Admin
                    && adm_user_role.community_id.unwrap() == role.community_id.unwrap())
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

    match admin_model::AdminModel::db_delete_by_id(conn, id) {
        Ok(_) => HttpResponse::Ok().json(HttpResponseObjectEmpty {
            error: false,
            message: "Admin deleted successfully".to_string(),
        }),
        Err(e) => HttpResponse::InternalServerError().json(HttpResponseObjectEmptyError {
            error: true,
            message: format!("Error deleting admin: {}", e),
        }),
    }
}
