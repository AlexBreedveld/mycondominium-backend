use super::*;

#[utoipa::path(
    post,
    tag = "Incident",
    path = "/new",
    request_body = incident_model::IncidentModelNew,
    responses (
        (status = 200, description = "Incident added successfully", body = HttpResponseObjectEmptyEntity),
        (status = 400, description = "Error adding Incident", body = HttpResponseObjectEmptyError),
        (status = 401, description = "Unauthorized", body = HttpResponseObjectEmptyError),
        (status = 500, description = "Error adding Incident", body = HttpResponseObjectEmptyError),
    ),
    security(
        ("Token" = [])
    )
)]
pub async fn new_incident(
    body: web::Json<incident_model::IncidentModelNew>,
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
        || (role.role == UserRoles::Admin && role.community_id == resident.role.community_id)
        || (role.role == UserRoles::Resident && role.community_id == resident.role.community_id)
        || (role.role == UserRoles::Resident && role.user_id == resident.user.id))
    {
        return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
            error: true,
            message: "Unauthorized".to_string(),
        });
    }

    if let Err(validation_errors) = body.validate() {
        return HttpResponse::BadRequest().json(validation_errors);
    }

    let body_clone = body.clone();

    let mut new_obj = incident_model::IncidentModel {
        id: incident_model::IncidentModel::new_id(conn),
        resident_id: body_clone.resident_id,
        community_id: body_clone.community_id,
        name: body_clone.name,
        description: body_clone.description,
        status: body_clone.status,
        report_date: chrono::Utc::now().naive_utc(),
        resolution_date: body_clone.resolution_date,
        notes: body_clone.notes,
    };

    let body = body.clone();

    if role.role == UserRoles::Resident {
        new_obj = incident_model::IncidentModel {
            id: incident_model::IncidentModel::new_id(conn),
            resident_id: body.resident_id,
            community_id: body.community_id,
            name: body.name,
            description: body.description,
            status: incident_model::IncidentStatus::Reported,
            report_date: chrono::Utc::now().naive_utc(),
            resolution_date: None,
            notes: None,
        };
    }

    match new_obj.db_insert(conn) {
        Ok(_) => (),
        Err(e) => {
            log::error!("Error creating Incident: {}", e);
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmptyError {
                error: true,
                message: format!("Error creating Incident: {}", e),
            });
        }
    };

    HttpResponse::Ok().json(HttpResponseObjectEmptyEntity {
        error: false,
        message: "Incident created successfully".to_string(),
        entity_id: Some(new_obj.id),
    })
}

#[utoipa::path(
    put,
    tag = "Incident",
    path = "/update/{id}",
    request_body = incident_model::IncidentModelNew,
    params(
        ("id" = Uuid, Path, description = "Incident ID"),
    ),
    responses (
        (status = 200, description = "Incident updated successfully", body = HttpResponseObjectEmpty),
        (status = 400, description = "Error updating Incident", body = HttpResponseObjectEmptyError),
        (status = 401, description = "Unauthorized", body = HttpResponseObjectEmptyError),
        (status = 500, description = "Internal server error", body = HttpResponseObjectEmptyError),
    ),
    security(
        ("Token" = [])
    )
)]
pub async fn update_incident(
    id: web::Path<String>,
    body: web::Json<incident_model::IncidentModelNew>,
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
                message: "Invalid Incident ID format".to_string(),
            });
        }
    };

    let curr_obj = match incident_model::IncidentModel::db_read_by_id(conn, id) {
        Ok(res) => res,
        Err(e) => {
            log::error!("Error updating Incident: {}", e);
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmpty {
                error: true,
                message: "Error updating Incident".to_string(),
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
        || (role.role == UserRoles::Admin && role.community_id == resident.role.community_id)
        || (role.role == UserRoles::Resident && role.community_id == resident.role.community_id)
        || (role.role == UserRoles::Resident && role.user_id == resident.user.id))
    {
        return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
            error: true,
            message: "Unauthorized".to_string(),
        });
    }

    if let Err(validation_errors) = body.validate() {
        return HttpResponse::BadRequest().json(validation_errors);
    }

    let body_clone = body.clone();

    let mut new_obj = incident_model::IncidentModel {
        id: curr_obj.id,
        resident_id: curr_obj.resident_id,
        community_id: curr_obj.community_id,
        name: body_clone.name,
        description: body_clone.description,
        status: body_clone.status,
        report_date: curr_obj.report_date,
        resolution_date: body_clone.resolution_date,
        notes: body_clone.notes,
    };

    let body = body.clone();

    if role.role == UserRoles::Resident {
        new_obj = incident_model::IncidentModel {
            id: curr_obj.id,
            resident_id: curr_obj.resident_id,
            community_id: curr_obj.community_id,
            name: body.name,
            description: body.description,
            status: curr_obj.status,
            report_date: curr_obj.report_date,
            resolution_date: curr_obj.resolution_date,
            notes: curr_obj.notes,
        };
    }

    match new_obj.db_update(conn) {
        Ok(_) => HttpResponse::Ok().json(HttpResponseObjectEmpty {
            error: false,
            message: "Incident updated successfully".to_string(),
        }),
        Err(e) => {
            log::error!("Error updating Incident: {}", e);
            HttpResponse::Ok().json(HttpResponseObjectEmpty {
                error: true,
                message: format!("Error updating Incident: {}", e),
            })
        }
    }
}

#[utoipa::path(
    delete,
    tag = "Incident",
    path = "/delete/{id}",
    params(
        ("id" = Uuid, Path, description = "Incident ID"),
    ),
    responses (
        (status = 200, description = "Incident deleted successfully", body = HttpResponseObjectEmpty),
        (status = 400, description = "Invalid Incident ID format", body = HttpResponseObjectEmptyError),
        (status = 401, description = "Unauthorized", body = HttpResponseObjectEmptyError),
        (status = 500, description = "Error deleting Incident", body = HttpResponseObjectEmptyError),
    ),
    security(
        ("Token" = [])
    )
)]
pub async fn delete_incident(
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
                message: "Invalid Incident ID format".to_string(),
            });
        }
    };

    let curr_obj = match incident_model::IncidentModel::db_read_by_id(conn, id) {
        Ok(user_req) => user_req,
        Err(e) => {
            log::error!("Error deleting Incident: {}", e);
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmpty {
                error: true,
                message: "Error deleting Incident".to_string(),
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

    match incident_model::IncidentModel::db_delete_by_id(conn, id) {
        Ok(_) => HttpResponse::Ok().json(HttpResponseObjectEmpty {
            error: false,
            message: "Incident deleted successfully".to_string(),
        }),
        Err(e) => {
            log::error!("Error deleting Incident: {}", e);
            HttpResponse::Ok().json(HttpResponseObjectEmpty {
                error: true,
                message: format!("Error deleting Incident: {}", e),
            })
        }
    }
}
