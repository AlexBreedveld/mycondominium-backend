use super::*;

#[utoipa::path(
    post,
    tag = "Community",
    path = "/new",
    request_body = community_model::CommunityModelNew,
    responses (
        (status = 200, description = "Community added successfully", body = HttpResponseObjectEmptyEntity),
        (status = 400, description = "Error adding Community", body = HttpResponseObjectEmptyError),
        (status = 401, description = "Unauthorized", body = HttpResponseObjectEmptyError),
        (status = 500, description = "Error adding Community", body = HttpResponseObjectEmptyError),
    ),
    security(
        ("Token" = [])
    )
)]
pub async fn new_community(body: web::Json<community_model::CommunityModelNew>, req: HttpRequest) -> HttpResponse {
    let conn = &mut establish_connection_pg();

    let body = body.into_inner();

    match authenticate_user(req.clone(), conn) {
        Ok((role, claims, token)) => {
            if role.role != UserRoles::Root {
                return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
                    error: true,
                    message: "Unauthorized".to_string(),
                })
            }
        }
        Err(_) => {
            return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Unauthorized".to_string(),
            })
        }
    }

    if let Err(validation_errors) = body.validate() {
        return HttpResponse::BadRequest().json(validation_errors);
    }

    let new_obj = community_model::CommunityModel {
        id: community_model::CommunityModel::new_id(conn),
        name: body.name,
        short_name: body.short_name,
        address: body.address,
        created_at: chrono::Utc::now().naive_utc(),
        updated_at: chrono::Utc::now().naive_utc(),
    };

    match new_obj.db_insert(conn) {
        Ok(_) => (),
        Err(e) => {
            log::error!("Error creating Community: {}", e);
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Error creating Community".to_string(),
            });
        }
    };

    HttpResponse::Ok().json(HttpResponseObjectEmptyEntity {
        error: false,
        message: "Community created successfully".to_string(),
        entity_id: Some(new_obj.id),
    })
}

#[utoipa::path(
    put,
    tag = "Community",
    path = "/update/{id}",
    request_body = community_model::CommunityModelNew,
    params(
        ("id" = Uuid, Path, description = "Community ID"),
    ),
    responses (
        (status = 200, description = "Community updated successfully", body = HttpResponseObjectEmpty),
        (status = 400, description = "Error updating Community", body = HttpResponseObjectEmptyError),
        (status = 401, description = "Unauthorized", body = HttpResponseObjectEmptyError),
        (status = 500, description = "Internal server error", body = HttpResponseObjectEmptyError),
    ),
    security(
        ("Token" = [])
    )
)]
pub async fn update_community(
    id: web::Path<String>,
    body: web::Json<community_model::CommunityModelNew>,
    req: HttpRequest
) -> HttpResponse {
    let conn = &mut establish_connection_pg();
    let body = body.into_inner();

    let id = match Uuid::parse_str(&id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return HttpResponse::BadRequest().json(HttpResponseObjectEmpty {
                error: true,
                message: "Invalid Community ID format".to_string(),
            });
        }
    };

    match authenticate_user(req.clone(), conn) {
        Ok((role, claims, token)) => {
            match role.role {
                UserRoles::Root => (),
                UserRoles::Admin => {
                    if role.community_id != Some(id) {
                        return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
                            error: true,
                            message: "Unauthorized".to_string(),
                        })
                    }
                },
                _ => {
                    return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
                        error: true,
                        message: "Unauthorized".to_string(),
                    })
                }
            }
        }
        Err(_) => {
            return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Unauthorized".to_string(),
            })
        }
    }

    if let Err(validation_errors) = body.validate() {
        return HttpResponse::BadRequest().json(validation_errors);
    }

    let curr_obj = match community_model::CommunityModel::db_read_by_id(conn, id) {
        Ok(ent_req) => ent_req,
        Err(e) => {
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmpty {
                error: true,
                message: format!("Error getting Community: {}", e),
            });
        }
    };

    let new_obj = community_model::CommunityModel {
        id: curr_obj.id,
        name: body.name,
        short_name: body.short_name,
        address: body.address,
        created_at: curr_obj.created_at,
        updated_at: chrono::Utc::now().naive_utc(),
    };

    match new_obj.db_update(conn) {
        Ok(_) => HttpResponse::Ok().json(HttpResponseObjectEmpty {
            error: false,
            message: "Community updated successfully".to_string(),
        }),
        Err(e) => HttpResponse::Ok().json(HttpResponseObjectEmpty {
            error: true,
            message: format!("Error updating Community: {}", e),
        }),
    }
}

#[utoipa::path(
    delete,
    tag = "Community",
    path = "/delete/{id}",
    params(
        ("id" = Uuid, Path, description = "Community ID"),
    ),
    responses (
        (status = 200, description = "Community deleted successfully", body = HttpResponseObjectEmpty),
        (status = 400, description = "Invalid Community ID format", body = HttpResponseObjectEmptyError),
        (status = 401, description = "Unauthorized", body = HttpResponseObjectEmptyError),
        (status = 500, description = "Error deleting Community", body = HttpResponseObjectEmptyError),
    ),
    security(
        ("Token" = [])
    )
)]
pub async fn delete_community(id: web::Path<String>, req: HttpRequest) -> HttpResponse {
    let conn = &mut establish_connection_pg();

    let id = match Uuid::parse_str(&id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return HttpResponse::BadRequest().json(HttpResponseObjectEmpty {
                error: true,
                message: "Invalid Community ID format".to_string(),
            });
        }
    };

    match authenticate_user(req.clone(), conn) {
        Ok((role, claims, token)) => {
            match role.role {
                UserRoles::Root => (),
                _ => {
                    return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
                        error: true,
                        message: "Unauthorized".to_string(),
                    })
                }
            }
        }
        Err(_) => {
            return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Unauthorized".to_string(),
            })
        }
    }

    match community_model::CommunityModel::db_delete_by_id(conn, id) {
        Ok(_) => HttpResponse::Ok().json(HttpResponseObjectEmpty {
            error: false,
            message: "Community deleted successfully".to_string(),
        }),
        Err(e) => HttpResponse::InternalServerError().json(HttpResponseObjectEmptyError {
            error: true,
            message: format!("Error deleting Community: {}", e),
        }),
    }
}
