use super::*;

#[utoipa::path(
    get,
    tag = "MaintenanceSchedule",
    path = "/list",
    params(
        ("page" = Option<i64>, Query, description = "Page number for pagination (default: 1)"),
        ("per_page" = Option<i64>, Query, description = "Number of items per page for pagination (default: 10)"),
    ),
    responses(
        (status = 200, description = "Got Maintenance Schedules successfully", body = MaintenanceScheduleListHttpResponse, headers(
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
pub async fn get_maintenance_schedules(
    query: web::Query<PaginationParams>,
    req: HttpRequest,
    conf: web::Data<Arc<MyCondominiumConfig>>,
) -> HttpResponse {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(10);
    let offset = (page - 1) * per_page;

    let conn = &mut establish_connection_pg(&conf);

    let (role, _claims, _token) = match authenticate_user(req.clone(), conn, conf) {
        Ok((role, claims, token)) => (role, claims, token),
        Err(_) => {
            return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Unauthorized".to_string(),
            });
        }
    };

    let total_items = match role.role {
        UserRoles::Root => {
            match maintenance_schedule_model::MaintenanceScheduleModel::db_count_all(conn) {
                Ok(count) => count,
                Err(e) => {
                    log::error!("Error getting total items: {}", e);
                    return HttpResponse::InternalServerError().json(HttpResponseObjectEmpty {
                        error: true,
                        message: "Error getting total items".to_string(),
                    });
                }
            }
        }
        UserRoles::Admin => {
            match maintenance_schedule_model::MaintenanceScheduleModel::table()
                .filter(maintenance_schedules::community_id.eq(role.community_id))
                .count()
                .get_result::<i64>(conn)
            {
                Ok(count) => count,
                Err(e) => {
                    log::error!("Error getting total items: {}", e);
                    return HttpResponse::InternalServerError().json(HttpResponseObjectEmpty {
                        error: true,
                        message: format!("Error getting total items: {}", e),
                    });
                }
            }
        }
        _ => {
            return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Unauthorized".to_string(),
            });
        }
    };

    match maintenance_schedule_model::MaintenanceScheduleModel::db_read_all_matching_community_by_range(
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
                    message: "Got Maintenance Schedules successfully".to_string(),
                    object: Some(res),
                })
        }
        Err(e) => {
            log::error!("Error getting Maintenance Schedules: {}", e);
            HttpResponse::InternalServerError().json(HttpResponseObjectEmpty {
                error: true,
                message: format!("Error getting Maintenance Schedules: {}", e),
            })
        },
    }
}

#[utoipa::path(
    get,
    tag = "MaintenanceSchedule",
    path = "/count/{status}",
    params(
        ("status" = maintenance_schedule_model::MaintenanceScheduleStatus, Path, description = "Maintenance Schedule Status"),
    ),
    responses(
        (status = 200, description = "Got Maintenance Schedules successfully", body = HttpResponseObject<i64>),
        (status = 401, description = "Unauthorized", body = HttpResponseObjectEmptyError),
        (status = 500, description = "Internal server error", body = HttpResponseObjectEmptyError)
    ),
    security(
        ("Token" = [])
    )
)]
pub async fn count_maintenance_schedule(
    status: web::Path<maintenance_schedule_model::MaintenanceScheduleStatus>,
    req: HttpRequest,
    conf: web::Data<Arc<MyCondominiumConfig>>,
) -> HttpResponse {
    let conn = &mut establish_connection_pg(&conf);

    let (role, _claims, _token) = match authenticate_user(req.clone(), conn, conf) {
        Ok((role, claims, token)) => (role, claims, token),
        Err(_) => {
            return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Unauthorized".to_string(),
            });
        }
    };

    match maintenance_schedule_model::MaintenanceScheduleModel::db_count_all_matching_community(
        role,
        status.into_inner(),
        conn,
    ) {
        Ok(res) => HttpResponse::Ok().json(HttpResponseObject {
            error: false,
            message: "Got Maintenance Schedules successfully".to_string(),
            object: Some(res),
        }),
        Err(e) => {
            log::error!("Error getting Maintenance Schedules: {}", e);
            HttpResponse::InternalServerError().json(HttpResponseObjectEmpty {
                error: true,
                message: format!("Error getting Maintenance Schedules: {}", e),
            })
        }
    }
}

#[utoipa::path(
    get,
    tag = "MaintenanceSchedule",
    path = "/get/{id}",
    params(
        ("id" = Uuid, Path, description = "Maintenance Schedule ID"),
    ),
    responses(
        (status = 200, description = "Got Maintenance Schedule successfully", body = MaintenanceScheduleGetHttpResponse),
        (status = 400, description = "Invalid Maintenance Schedule ID format or Maintenance Schedule ID is required", body = HttpResponseObjectEmptyError),
        (status = 401, description = "Unauthorized", body = HttpResponseObjectEmptyError),
        (status = 500, description = "Internal server error", body = HttpResponseObjectEmptyError)
    ),
    security(
        ("Token" = [])
    )
)]
pub async fn get_maintenance_schedule_by_id(
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
                message: "Invalid Maintenance Schedule ID format".to_string(),
            });
        }
    };

    let role = match authenticate_user(req.clone(), conn, conf) {
        Ok((role, _claims, _token)) => {
            if role.role == UserRoles::Root
                || role.role == UserRoles::Admin
                || role.role == UserRoles::Resident
            {
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

    match maintenance_schedule_model::MaintenanceScheduleModel::db_read_by_id_matching_community(
        role, conn, id,
    ) {
        Ok(user_req) => HttpResponse::Ok().json(HttpResponseObject {
            error: false,
            message: "Got Maintenance Schedule successfully".to_string(),
            object: Some(user_req),
        }),
        Err(e) => {
            log::error!("Error getting Maintenance Schedule: {}", e);
            HttpResponse::InternalServerError().json(HttpResponseObjectEmpty {
                error: true,
                message: format!("Error getting Maintenance Schedule: {}", e),
            })
        }
    }
}
