use super::*;

#[utoipa::path(
    post,
    tag = "Election",
    path = "/new",
    request_body = election_model::ElectionModelNew,
    responses (
        (status = 200, description = "Election added successfully", body = HttpResponseObjectEmptyEntity),
        (status = 400, description = "Error adding Election", body = HttpResponseObjectEmptyError),
        (status = 401, description = "Unauthorized", body = HttpResponseObjectEmptyError),
        (status = 500, description = "Error adding Election", body = HttpResponseObjectEmptyError),
    ),
    security(
        ("Token" = [])
    )
)]
pub async fn new_election(
    body: web::Json<election_model::ElectionModelNew>,
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

    let new_obj = election_model::ElectionModel {
        id: election_model::ElectionModel::new_id(conn),
        community_id: body.community_id,
        title: body.title,
        description: body.description,
        start_date: body.start_date,
        end_date: body.end_date,
        created_at: chrono::Utc::now().naive_utc(),
    };

    match new_obj.db_insert(conn) {
        Ok(_) => (),
        Err(e) => {
            log::error!("Error creating Election: {}", e);
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmptyError {
                error: true,
                message: format!("Error creating Election: {}", e),
            });
        }
    };

    HttpResponse::Ok().json(HttpResponseObjectEmptyEntity {
        error: false,
        message: "Election created successfully".to_string(),
        entity_id: Some(new_obj.id),
    })
}

#[utoipa::path(
    put,
    tag = "Election",
    path = "/update/{id}",
    request_body = election_model::ElectionModelNew,
    params(
        ("id" = Uuid, Path, description = "Election ID"),
    ),
    responses (
        (status = 200, description = "Election updated successfully", body = HttpResponseObjectEmpty),
        (status = 400, description = "Error updating Election", body = HttpResponseObjectEmptyError),
        (status = 401, description = "Unauthorized", body = HttpResponseObjectEmptyError),
        (status = 500, description = "Internal server error", body = HttpResponseObjectEmptyError),
    ),
    security(
        ("Token" = [])
    )
)]
pub async fn update_election(
    id: web::Path<String>,
    body: web::Json<election_model::ElectionModelNew>,
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

    let id = match Uuid::parse_str(&id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return HttpResponse::BadRequest().json(HttpResponseObjectEmpty {
                error: true,
                message: "Invalid Election ID format".to_string(),
            });
        }
    };

    let curr_obj = match election_model::ElectionModel::db_read_by_id(conn, id) {
        Ok(res) => res,
        Err(e) => {
            log::error!("Error updating Election: {}", e);
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmpty {
                error: true,
                message: "Error updating Election".to_string(),
            });
        }
    };

    match role.role {
        UserRoles::Root => {}
        UserRoles::Admin => {
            if role.community_id != Some(curr_obj.community_id)
                || role.community_id != Some(body.community_id)
            {
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

    if let Err(validation_errors) = body.validate() {
        return HttpResponse::BadRequest().json(validation_errors);
    }

    let new_obj = election_model::ElectionModel {
        id: curr_obj.id,
        community_id: body.community_id,
        title: body.title,
        description: body.description,
        start_date: body.start_date,
        end_date: body.end_date,
        created_at: curr_obj.created_at,
    };

    match new_obj.db_update(conn) {
        Ok(_) => HttpResponse::Ok().json(HttpResponseObjectEmpty {
            error: false,
            message: "Election updated successfully".to_string(),
        }),
        Err(e) => {
            log::error!("Error updating Election: {}", e);
            HttpResponse::Ok().json(HttpResponseObjectEmpty {
                error: true,
                message: format!("Error updating Election: {}", e),
            })
        }
    }
}

#[utoipa::path(
    delete,
    tag = "Election",
    path = "/delete/{id}",
    params(
        ("id" = Uuid, Path, description = "Election ID"),
    ),
    responses (
        (status = 200, description = "Election deleted successfully", body = HttpResponseObjectEmpty),
        (status = 400, description = "Invalid Election ID format", body = HttpResponseObjectEmptyError),
        (status = 401, description = "Unauthorized", body = HttpResponseObjectEmptyError),
        (status = 500, description = "Error deleting Election", body = HttpResponseObjectEmptyError),
    ),
    security(
        ("Token" = [])
    )
)]
pub async fn delete_election(
    id: web::Path<String>,
    req: HttpRequest,
    conf: web::Data<Arc<MyCondominiumConfig>>,
) -> HttpResponse {
    let conn = &mut establish_connection_pg(&conf);

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

    let id = match Uuid::parse_str(&id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return HttpResponse::BadRequest().json(HttpResponseObjectEmpty {
                error: true,
                message: "Invalid Election ID format".to_string(),
            });
        }
    };

    let curr_obj = match election_model::ElectionModel::db_read_by_id(conn, id) {
        Ok(user_req) => user_req,
        Err(e) => {
            log::error!("Error deleting Election: {}", e);
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmpty {
                error: true,
                message: "Error deleting Election".to_string(),
            });
        }
    };

    if !(role.role == UserRoles::Root
        || (role.role == UserRoles::Admin && role.community_id == Some(curr_obj.community_id)))
    {
        return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
            error: true,
            message: "Unauthorized".to_string(),
        });
    }

    match election_model::ElectionModel::db_delete_by_id(conn, id) {
        Ok(_) => HttpResponse::Ok().json(HttpResponseObjectEmpty {
            error: false,
            message: "Election deleted successfully".to_string(),
        }),
        Err(e) => {
            log::error!("Error deleting Election: {}", e);
            HttpResponse::Ok().json(HttpResponseObjectEmpty {
                error: true,
                message: format!("Error deleting Election: {}", e),
            })
        }
    }
}
