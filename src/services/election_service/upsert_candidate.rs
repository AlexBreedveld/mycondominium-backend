use super::*;

#[utoipa::path(
    post,
    tag = "Election",
    path = "/candidate/new",
    request_body = election_model::ElectionCandidatesModelNew,
    responses (
        (status = 200, description = "Election candidate added successfully", body = HttpResponseObjectEmptyEntity),
        (status = 400, description = "Error adding Election candidate", body = HttpResponseObjectEmptyError),
        (status = 401, description = "Unauthorized", body = HttpResponseObjectEmptyError),
        (status = 500, description = "Error adding Election candidate", body = HttpResponseObjectEmptyError),
    ),
    security(
        ("Token" = [])
    )
)]
pub async fn new_election_candidate(
    body: web::Json<election_model::ElectionCandidatesModelNew>,
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

    let election = match election_model::ElectionModel::db_read_by_id(conn, body.election_id) {
        Ok(elec) => elec,
        Err(_) => {
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Error while getting election".to_string(),
            });
        }
    };

    if !(role.role == UserRoles::Root
        || (role.role == UserRoles::Admin && role.community_id == Some(election.community_id)))
    {
        return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
            error: true,
            message: "Unauthorized".to_string(),
        });
    }

    if let Err(validation_errors) = body.validate() {
        return HttpResponse::BadRequest().json(validation_errors);
    }

    let new_obj = election_model::ElectionCandidatesModel {
        id: election_model::ElectionCandidatesModel::new_id(conn),
        election_id: body.election_id,
        name: body.name,
    };

    match new_obj.db_insert(conn) {
        Ok(_) => (),
        Err(e) => {
            log::error!("Error creating Election candidate: {}", e);
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmptyError {
                error: true,
                message: format!("Error creating Election candidate: {}", e),
            });
        }
    };

    HttpResponse::Ok().json(HttpResponseObjectEmptyEntity {
        error: false,
        message: "Election candidate created successfully".to_string(),
        entity_id: Some(new_obj.id),
    })
}

#[utoipa::path(
    put,
    tag = "Election",
    path = "/candidate/update/{id}",
    request_body = election_model::ElectionCandidatesModelNew,
    params(
        ("id" = Uuid, Path, description = "Election Candidate ID"),
    ),
    responses (
        (status = 200, description = "Election candidate updated successfully", body = HttpResponseObjectEmpty),
        (status = 400, description = "Error updating Election candidate", body = HttpResponseObjectEmptyError),
        (status = 401, description = "Unauthorized", body = HttpResponseObjectEmptyError),
        (status = 500, description = "Internal server error", body = HttpResponseObjectEmptyError),
    ),
    security(
        ("Token" = [])
    )
)]
pub async fn update_election_candidate(
    id: web::Path<String>,
    body: web::Json<election_model::ElectionCandidatesModelNew>,
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
                message: "Invalid Election Candidate ID format".to_string(),
            });
        }
    };

    let curr_obj = match election_model::ElectionCandidatesModel::db_read_by_id(conn, id) {
        Ok(res) => res,
        Err(e) => {
            log::error!("Error updating Election candidate: {}", e);
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmpty {
                error: true,
                message: "Error updating Election candidate".to_string(),
            });
        }
    };

    let election = match election_model::ElectionModel::db_read_by_id(conn, body.election_id) {
        Ok(elec) => elec,
        Err(_) => {
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Error while getting election".to_string(),
            });
        }
    };

    if !(role.role == UserRoles::Root
        || (role.role == UserRoles::Admin && role.community_id == Some(election.community_id)))
    {
        return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
            error: true,
            message: "Unauthorized".to_string(),
        });
    }

    if let Err(validation_errors) = body.validate() {
        return HttpResponse::BadRequest().json(validation_errors);
    }

    let new_obj = election_model::ElectionCandidatesModel {
        id: curr_obj.id,
        election_id: body.election_id,
        name: body.name,
    };

    match new_obj.db_update(conn) {
        Ok(_) => HttpResponse::Ok().json(HttpResponseObjectEmpty {
            error: false,
            message: "Election candidate updated successfully".to_string(),
        }),
        Err(e) => {
            log::error!("Error updating Election candidate: {}", e);
            HttpResponse::Ok().json(HttpResponseObjectEmpty {
                error: true,
                message: format!("Error updating Election candidate: {}", e),
            })
        }
    }
}

#[utoipa::path(
    delete,
    tag = "Election",
    path = "/candidate/delete/{id}",
    params(
        ("id" = Uuid, Path, description = "Election Candidate ID"),
    ),
    responses (
        (status = 200, description = "Election candidate deleted successfully", body = HttpResponseObjectEmpty),
        (status = 400, description = "Invalid Election Candidate ID format", body = HttpResponseObjectEmptyError),
        (status = 401, description = "Unauthorized", body = HttpResponseObjectEmptyError),
        (status = 500, description = "Error deleting Election candidate", body = HttpResponseObjectEmptyError),
    ),
    security(
        ("Token" = [])
    )
)]
pub async fn delete_election_candidate(
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
                message: "Invalid Election Candidate ID format".to_string(),
            });
        }
    };

    let candidate = match election_model::ElectionCandidatesModel::db_read_by_id(conn, id) {
        Ok(candidate) => candidate,
        Err(e) => {
            log::error!("Error deleting Election candidate: {}", e);
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmpty {
                error: true,
                message: "Error deleting Election candidate".to_string(),
            });
        }
    };

    let election = match election_model::ElectionModel::db_read_by_id(conn, candidate.election_id) {
        Ok(elec) => elec,
        Err(_) => {
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Error while getting election".to_string(),
            });
        }
    };

    if !(role.role == UserRoles::Root
        || (role.role == UserRoles::Admin && role.community_id == Some(election.community_id)))
    {
        return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
            error: true,
            message: "Unauthorized".to_string(),
        });
    }

    match election_model::ElectionCandidatesModel::db_delete_by_id(conn, id) {
        Ok(_) => HttpResponse::Ok().json(HttpResponseObjectEmpty {
            error: false,
            message: "Election candidate deleted successfully".to_string(),
        }),
        Err(e) => {
            log::error!("Error deleting Election candidate: {}", e);
            HttpResponse::Ok().json(HttpResponseObjectEmpty {
                error: true,
                message: format!("Error deleting Election candidate: {}", e),
            })
        }
    }
}
