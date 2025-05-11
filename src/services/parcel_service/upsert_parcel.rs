use super::*;

#[utoipa::path(
    post,
    tag = "Parcel",
    path = "/new",
    request_body = parcel_model::ParcelModelNew,
    responses (
        (status = 200, description = "Parcel added successfully", body = HttpResponseObjectEmptyEntity),
        (status = 400, description = "Error adding Parcel", body = HttpResponseObjectEmptyError),
        (status = 401, description = "Unauthorized", body = HttpResponseObjectEmptyError),
        (status = 500, description = "Error adding Parcel", body = HttpResponseObjectEmptyError),
    ),
    security(
        ("Token" = [])
    )
)]
pub async fn new_parcel(
    body: web::Json<parcel_model::ParcelModelNew>,
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

    let resident = match resident_model::ResidentModel::db_get_user(conn, body.resident_id) {
        Ok(resident) => resident,
        Err(_) => {
            return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Unauthorized".to_string(),
            });
        }
    };

    if !(role.role == UserRoles::Root
        || (role.role == UserRoles::Admin && role.community_id == resident.role.community_id))
    {
        return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
            error: true,
            message: "Unauthorized".to_string(),
        });
    }

    if let Err(validation_errors) = body.validate() {
        return HttpResponse::BadRequest().json(validation_errors);
    }

    let new_obj = parcel_model::ParcelModel {
        id: parcel_model::ParcelModel::new_id(conn),
        resident_id: body.resident_id,
        parcel_type: body.parcel_type,
        description: body.description,
        arrival_date: body.arrival_date,
        received: body.received,
        received_at: body.received_at,
    };

    match new_obj.db_insert(conn) {
        Ok(_) => (),
        Err(e) => {
            log::error!("Error creating Parcel: {}", e);
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmptyError {
                error: true,
                message: format!("Error creating Parcel: {}", e),
            });
        }
    };

    HttpResponse::Ok().json(HttpResponseObjectEmptyEntity {
        error: false,
        message: "Parcel created successfully".to_string(),
        entity_id: Some(new_obj.id),
    })
}

#[utoipa::path(
    put,
    tag = "Parcel",
    path = "/update/{id}",
    request_body = parcel_model::ParcelModelNew,
    params(
        ("id" = Uuid, Path, description = "Parcel ID"),
    ),
    responses (
        (status = 200, description = "Parcel updated successfully", body = HttpResponseObjectEmpty),
        (status = 400, description = "Error updating Parcel", body = HttpResponseObjectEmptyError),
        (status = 401, description = "Unauthorized", body = HttpResponseObjectEmptyError),
        (status = 500, description = "Internal server error", body = HttpResponseObjectEmptyError),
    ),
    security(
        ("Token" = [])
    )
)]
pub async fn update_parcel(
    id: web::Path<String>,
    body: web::Json<parcel_model::ParcelModelNew>,
    req: HttpRequest,
    conf: web::Data<Arc<MyCondominiumConfig>>,
) -> HttpResponse {
    let conn = &mut establish_connection_pg(&conf);
    let body = body.into_inner();

    let (role, _claims, _token) = match authenticate_user(req.clone(), conn, conf.clone()) {
        Ok((role, claims, token)) => (role, claims, token),
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
                message: "Invalid Parcel ID format".to_string(),
            });
        }
    };

    let curr_obj = match parcel_model::ParcelModel::db_read_by_id(conn, id) {
        Ok(res) => res,
        Err(e) => {
            log::error!("Error updating Parcel: {}", e);
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmpty {
                error: true,
                message: "Error updating Parcel".to_string(),
            });
        }
    };

    let resident = match resident_model::ResidentModel::db_get_user(conn, body.resident_id) {
        Ok(resident) => resident,
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
            if role.community_id != resident.role.community_id {
                return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
                    error: true,
                    message: "Unauthorized".to_string(),
                });
            }
        }
        UserRoles::Resident => {
            if role.user_id != resident.user.id {
                return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
                    error: true,
                    message: "Unauthorized".to_string(),
                });
            }
        }
    }

    if let Err(validation_errors) = body.validate() {
        return HttpResponse::BadRequest().json(validation_errors);
    }

    let mut new_obj = parcel_model::ParcelModel {
        id: curr_obj.id,
        resident_id: body.resident_id,
        parcel_type: body.parcel_type,
        description: body.description,
        arrival_date: body.arrival_date,
        received: body.received,
        received_at: body.received_at,
    };

    if role.role == UserRoles::Resident {
        new_obj = parcel_model::ParcelModel {
            id: curr_obj.id,
            resident_id: curr_obj.resident_id,
            parcel_type: curr_obj.parcel_type,
            description: curr_obj.description,
            arrival_date: curr_obj.arrival_date,
            received: body.received,
            received_at: body.received_at,
        };
    }

    match new_obj.db_update(conn) {
        Ok(_) => HttpResponse::Ok().json(HttpResponseObjectEmpty {
            error: false,
            message: "Parcel updated successfully".to_string(),
        }),
        Err(e) => {
            log::error!("Error updating Parcel: {}", e);
            HttpResponse::Ok().json(HttpResponseObjectEmpty {
                error: true,
                message: format!("Error updating Parcel: {}", e),
            })
        }
    }
}

#[utoipa::path(
    delete,
    tag = "Parcel",
    path = "/delete/{id}",
    params(
        ("id" = Uuid, Path, description = "Parcel ID"),
    ),
    responses (
        (status = 200, description = "Parcel deleted successfully", body = HttpResponseObjectEmpty),
        (status = 400, description = "Invalid Parcel ID format", body = HttpResponseObjectEmptyError),
        (status = 401, description = "Unauthorized", body = HttpResponseObjectEmptyError),
        (status = 500, description = "Error deleting Parcel", body = HttpResponseObjectEmptyError),
    ),
    security(
        ("Token" = [])
    )
)]
pub async fn delete_parcel(
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
                message: "Invalid Parcel ID format".to_string(),
            });
        }
    };

    let curr_obj = match parcel_model::ParcelModel::db_read_by_id(conn, id) {
        Ok(user_req) => user_req,
        Err(e) => {
            log::error!("Error deleting Parcel: {}", e);
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmpty {
                error: true,
                message: "Error deleting Parcel".to_string(),
            });
        }
    };

    let resident = match resident_model::ResidentModel::db_get_user(conn, curr_obj.resident_id) {
        Ok(resident) => resident,
        Err(_) => {
            return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Unauthorized".to_string(),
            });
        }
    };

    if !(role.role == UserRoles::Root
        || (role.role == UserRoles::Admin && role.community_id == resident.role.community_id))
    {
        return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
            error: true,
            message: "Unauthorized".to_string(),
        });
    }

    match parcel_model::ParcelModel::db_delete_by_id(conn, id) {
        Ok(_) => HttpResponse::Ok().json(HttpResponseObjectEmpty {
            error: false,
            message: "Parcel deleted successfully".to_string(),
        }),
        Err(e) => {
            log::error!("Error deleting Parcel: {}", e);
            HttpResponse::Ok().json(HttpResponseObjectEmpty {
                error: true,
                message: format!("Error deleting Parcel: {}", e),
            })
        }
    }
}
