use super::*;

#[utoipa::path(
    post,
    tag = "Reservation",
    path = "/new",
    request_body = reservation_model::ReservationModelNew,
    responses (
        (status = 200, description = "Reservation added successfully", body = HttpResponseObjectEmptyEntity),
        (status = 400, description = "Error adding Reservation", body = HttpResponseObjectEmptyError),
        (status = 401, description = "Unauthorized", body = HttpResponseObjectEmptyError),
        (status = 500, description = "Error adding Reservation", body = HttpResponseObjectEmptyError),
    ),
    security(
        ("Token" = [])
    )
)]
pub async fn new_reservation(
    body: web::Json<reservation_model::ReservationModelNew>,
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

    let comm_id = match body.get_community_id(conn) {
        Ok(comm_id) => comm_id,
        Err(e) => {
            log::error!("Error getting community id: {}", e);
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmptyError {
                error: true,
                message: format!("Error getting community id: {}", e),
            });
        }
    };

    match role.role {
        UserRoles::Root => {}
        UserRoles::Admin | UserRoles::Resident => {
            if comm_id != role.community_id.unwrap() {
                return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
                    error: true,
                    message: "Unauthorized".to_string(),
                });
            }
        }
    };

    if role.role == UserRoles::Resident {
        match user_model::UserModel::db_read_by_id(conn, role.user_id) {
            Ok(user) => {
                match resident_model::ResidentModel::db_read_by_id(conn, user.resident_id.unwrap())
                {
                    Ok(resident) => {
                        if resident.id != body.resident_id {
                            return HttpResponse::Unauthorized().json(
                                HttpResponseObjectEmptyError {
                                    error: true,
                                    message: "Unauthorized".to_string(),
                                },
                            );
                        }
                    }
                    Err(e) => {
                        log::error!("Error getting user: {}", e);
                        return HttpResponse::InternalServerError().json(
                            HttpResponseObjectEmptyError {
                                error: true,
                                message: "Error getting user".to_string(),
                            },
                        );
                    }
                }
            }
            Err(e) => {
                log::error!("Error getting user: {}", e);
                return HttpResponse::InternalServerError().json(HttpResponseObjectEmptyError {
                    error: true,
                    message: "Error getting user".to_string(),
                });
            }
        }
    }

    match body.check_for_overlap(conn) {
        Ok(_) => (),
        Err(e) => {
            if e.kind() == std::io::ErrorKind::AddrInUse {
                return HttpResponse::BadRequest().json(HttpResponseObjectEmptyError {
                    error: true,
                    message: format!("Your reservation overlaps with another: {}", e),
                });
            } else {
                log::error!("Error checking for overlap: {}", e);
                return HttpResponse::InternalServerError().json(HttpResponseObjectEmptyError {
                    error: true,
                    message: "Error checking for overlap".to_string(),
                });
            }
        }
    }

    let new_obj = reservation_model::ReservationModel {
        id: reservation_model::ReservationModel::new_id(conn),
        resident_id: body.resident_id,
        common_area_id: body.common_area_id,
        reservation_date: body.reservation_date,
        start_time: body.start_time,
        end_time: body.end_time,
        status: reservation_model::ReservationStatus::Reserved,
        created_at: chrono::Utc::now().naive_utc(),
        updated_at: chrono::Utc::now().naive_utc(),
    };

    match new_obj.db_insert(conn) {
        Ok(_) => (),
        Err(e) => {
            log::error!("Error creating Reservation: {}", e);
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Error creating Reservation".to_string(),
            });
        }
    };

    HttpResponse::Ok().json(HttpResponseObjectEmptyEntity {
        error: false,
        message: "Reservation created successfully".to_string(),
        entity_id: Some(new_obj.id),
    })
}

#[utoipa::path(
    put,
    tag = "Reservation",
    path = "/update/{id}",
    request_body = reservation_model::ReservationModelNew,
    params(
        ("id" = Uuid, Path, description = "Reservation ID"),
    ),
    responses (
        (status = 200, description = "Reservation updated successfully", body = HttpResponseObjectEmpty),
        (status = 400, description = "Error updating Reservation", body = HttpResponseObjectEmptyError),
        (status = 401, description = "Unauthorized", body = HttpResponseObjectEmptyError),
        (status = 500, description = "Internal server error", body = HttpResponseObjectEmptyError),
    ),
    security(
        ("Token" = [])
    )
)]
pub async fn update_reservation(
    id: web::Path<String>,
    body: web::Json<reservation_model::ReservationModelNew>,
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

    let curr_obj = match reservation_model::ReservationModel::db_read_by_id(conn, id) {
        Ok(res) => res,
        Err(e) => {
            log::error!("Error updating Reservation: {}", e);
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmpty {
                error: true,
                message: "Error updating Reservation".to_string(),
            });
        }
    };

    let comm_id = match body.get_community_id(conn) {
        Ok(comm_id) => comm_id,
        Err(e) => {
            log::error!("Error getting community id: {}", e);
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmptyError {
                error: true,
                message: format!("Error getting community id: {}", e),
            });
        }
    };

    match role.role {
        UserRoles::Root => {}
        UserRoles::Admin | UserRoles::Resident => {
            if role.community_id.unwrap() != comm_id {
                return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
                    error: true,
                    message: "Unauthorized".to_string(),
                });
            }
        }
    }

    let new_obj = reservation_model::ReservationModel {
        id: curr_obj.id,
        resident_id: body.resident_id,
        common_area_id: body.common_area_id,
        reservation_date: body.reservation_date,
        start_time: body.start_time,
        end_time: body.end_time,
        status: curr_obj.status,
        created_at: curr_obj.created_at,
        updated_at: chrono::Utc::now().naive_utc(),
    };

    if role.role == UserRoles::Resident {
        match user_model::UserModel::db_read_by_id(conn, role.user_id) {
            Ok(user) => {
                match resident_model::ResidentModel::db_read_by_id(conn, user.resident_id.unwrap())
                {
                    Ok(resident) => {
                        if resident.id != curr_obj.resident_id {
                            return HttpResponse::Unauthorized().json(
                                HttpResponseObjectEmptyError {
                                    error: true,
                                    message: "Unauthorized".to_string(),
                                },
                            );
                        }
                    }
                    Err(e) => {
                        log::error!("Error getting user: {}", e);
                        return HttpResponse::InternalServerError().json(
                            HttpResponseObjectEmptyError {
                                error: true,
                                message: "Error getting user".to_string(),
                            },
                        );
                    }
                }
            }
            Err(e) => {
                log::error!("Error getting user: {}", e);
                return HttpResponse::InternalServerError().json(HttpResponseObjectEmptyError {
                    error: true,
                    message: "Error getting user".to_string(),
                });
            }
        }
    }

    match new_obj.check_for_overlap(conn) {
        Ok(_) => (),
        Err(e) => {
            if e.kind() == std::io::ErrorKind::AddrInUse {
                return HttpResponse::BadRequest().json(HttpResponseObjectEmptyError {
                    error: true,
                    message: format!("Your reservation overlaps with another: {}", e),
                });
            } else {
                log::error!("Error checking for overlap: {}", e);
                return HttpResponse::InternalServerError().json(HttpResponseObjectEmptyError {
                    error: true,
                    message: "Error checking for overlap".to_string(),
                });
            }
        }
    }

    match new_obj.db_update(conn) {
        Ok(_) => HttpResponse::Ok().json(HttpResponseObjectEmpty {
            error: false,
            message: "Reservation updated successfully".to_string(),
        }),
        Err(e) => {
            log::error!("Error updating Reservation: {}", e);
            HttpResponse::Ok().json(HttpResponseObjectEmpty {
                error: true,
                message: format!("Error updating Reservation: {}", e),
            })
        }
    }
}

#[utoipa::path(
    delete,
    tag = "Reservation",
    path = "/delete/{id}",
    params(
        ("id" = Uuid, Path, description = "Reservation ID"),
    ),
    responses (
        (status = 200, description = "Reservation deleted successfully", body = HttpResponseObjectEmpty),
        (status = 400, description = "Invalid Reservation ID format", body = HttpResponseObjectEmptyError),
        (status = 401, description = "Unauthorized", body = HttpResponseObjectEmptyError),
        (status = 500, description = "Error deleting Reservation", body = HttpResponseObjectEmptyError),
    ),
    security(
        ("Token" = [])
    )
)]
pub async fn delete_reservation(
    id: web::Path<String>,
    req: HttpRequest,
    conf: web::Data<Arc<MyCondominiumConfig>>,
) -> HttpResponse {
    let conn = &mut establish_connection_pg(&conf);

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

    let curr_obj = match reservation_model::ReservationModel::db_read_by_id(conn, id) {
        Ok(res) => res,
        Err(e) => {
            log::error!("Error updating Reservation: {}", e);
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmpty {
                error: true,
                message: "Error updating Reservation".to_string(),
            });
        }
    };

    let comm_id = match curr_obj.get_community_id(conn) {
        Ok(comm_id) => comm_id,
        Err(e) => {
            log::error!("Error getting community id: {}", e);
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Error getting community id".to_string(),
            });
        }
    };

    if role.role == UserRoles::Resident {
        match user_model::UserModel::db_read_by_id(conn, role.user_id) {
            Ok(user) => {
                match resident_model::ResidentModel::db_read_by_id(conn, user.resident_id.unwrap())
                {
                    Ok(resident) => {
                        if resident.id != curr_obj.resident_id {
                            return HttpResponse::Unauthorized().json(
                                HttpResponseObjectEmptyError {
                                    error: true,
                                    message: "Unauthorized".to_string(),
                                },
                            );
                        }
                    }
                    Err(e) => {
                        log::error!("Error getting user: {}", e);
                        return HttpResponse::InternalServerError().json(
                            HttpResponseObjectEmptyError {
                                error: true,
                                message: "Error getting user".to_string(),
                            },
                        );
                    }
                }
            }
            Err(e) => {
                log::error!("Error getting user: {}", e);
                return HttpResponse::InternalServerError().json(HttpResponseObjectEmptyError {
                    error: true,
                    message: "Error getting user".to_string(),
                });
            }
        }
    }

    match role.role {
        UserRoles::Root => {}
        UserRoles::Admin | UserRoles::Resident => {
            if role.community_id.unwrap() != comm_id {
                return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
                    error: true,
                    message: "Unauthorized".to_string(),
                });
            }
        }
    }

    match reservation_model::ReservationModel::db_delete_by_id(conn, id) {
        Ok(_) => HttpResponse::Ok().json(HttpResponseObjectEmpty {
            error: false,
            message: "Reservation deleted successfully".to_string(),
        }),
        Err(e) => {
            log::error!("Error deleting Reservation: {}", e);
            HttpResponse::Ok().json(HttpResponseObjectEmpty {
                error: true,
                message: format!("Error deleting Reservation: {}", e),
            })
        }
    }
}
