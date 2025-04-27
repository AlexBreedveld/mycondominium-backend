use super::*;
use crate::utilities::user_utils::check_email_exist;

#[utoipa::path(
    post,
    tag = "MaintenanceSchedule",
    path = "/new",
    request_body = maintenance_schedule_model::MaintenanceScheduleModelNew,
    responses (
        (status = 200, description = "Maintenance Schedule added successfully", body = HttpResponseObjectEmptyEntity),
        (status = 400, description = "Error adding Maintenance Schedule", body = HttpResponseObjectEmptyError),
        (status = 401, description = "Unauthorized", body = HttpResponseObjectEmptyError),
        (status = 500, description = "Error adding Maintenance Schedule", body = HttpResponseObjectEmptyError),
    ),
    security(
        ("Token" = [])
    )
)]
pub async fn new_maintenance_schedule(
    body: web::Json<maintenance_schedule_model::MaintenanceScheduleModelNew>,
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

    let new_obj = maintenance_schedule_model::MaintenanceScheduleModel {
        id: maintenance_schedule_model::MaintenanceScheduleModel::new_id(conn),
        community_id: body.community_id,
        description: body.description,
        scheduled_date: body.scheduled_date,
        status: body.status,
        details: body.details,
        created_at: chrono::Utc::now().naive_utc(),
        updated_at: chrono::Utc::now().naive_utc(),
    };

    match new_obj.db_insert(conn) {
        Ok(_) => (),
        Err(e) => {
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmptyError {
                error: true,
                message: format!("Error creating Maintenance Schedule: {}", e),
            });
        }
    };

    HttpResponse::Ok().json(HttpResponseObjectEmptyEntity {
        error: false,
        message: "Maintenance Schedule created successfully".to_string(),
        entity_id: Some(new_obj.id),
    })
}

#[utoipa::path(
    put,
    tag = "MaintenanceSchedule",
    path = "/update/{id}",
    request_body = maintenance_schedule_model::MaintenanceScheduleModelNew,
    params(
        ("id" = Uuid, Path, description = "Maintenance Schedule ID"),
    ),
    responses (
        (status = 200, description = "Maintenance Schedule updated successfully", body = HttpResponseObjectEmpty),
        (status = 400, description = "Error updating Maintenance Schedule", body = HttpResponseObjectEmptyError),
        (status = 401, description = "Unauthorized", body = HttpResponseObjectEmptyError),
        (status = 500, description = "Internal server error", body = HttpResponseObjectEmptyError),
    ),
    security(
        ("Token" = [])
    )
)]
pub async fn update_maintenance_schedule(
    id: web::Path<String>,
    body: web::Json<maintenance_schedule_model::MaintenanceScheduleModelNew>,
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

    let id = match Uuid::parse_str(&id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return HttpResponse::BadRequest().json(HttpResponseObjectEmpty {
                error: true,
                message: "Invalid Resident ID format".to_string(),
            });
        }
    };

    let curr_obj =
        match maintenance_schedule_model::MaintenanceScheduleModel::db_read_by_id(conn, id) {
            Ok(user_req) => user_req,
            Err(e) => {
                return HttpResponse::InternalServerError().json(HttpResponseObjectEmpty {
                    error: true,
                    message: "Error updating Maintenance Schedule".to_string(),
                });
            }
        };

    if !(role.role == UserRoles::Root
        || (role.role == UserRoles::Admin
            && role.community_id == body.community_id
            && role.community_id == curr_obj.community_id))
    {
        return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
            error: true,
            message: "Unauthorized".to_string(),
        });
    }

    if let Err(validation_errors) = body.validate() {
        return HttpResponse::BadRequest().json(validation_errors);
    }

    let new_obj = maintenance_schedule_model::MaintenanceScheduleModel {
        id: curr_obj.id,
        community_id: body.community_id,
        description: body.description,
        scheduled_date: body.scheduled_date,
        status: body.status,
        details: body.details,
        created_at: curr_obj.created_at,
        updated_at: chrono::Utc::now().naive_utc(),
    };

    match new_obj.db_update(conn) {
        Ok(_) => HttpResponse::Ok().json(HttpResponseObjectEmpty {
            error: false,
            message: "Maintenance Schedule updated successfully".to_string(),
        }),
        Err(e) => HttpResponse::Ok().json(HttpResponseObjectEmpty {
            error: true,
            message: format!("Error creating Maintenance Schedule: {}", e),
        }),
    }
}

#[utoipa::path(
    delete,
    tag = "MaintenanceSchedule",
    path = "/delete/{id}",
    params(
        ("id" = Uuid, Path, description = "Maintenance Schedule ID"),
    ),
    responses (
        (status = 200, description = "Maintenance Schedule deleted successfully", body = HttpResponseObjectEmpty),
        (status = 400, description = "Invalid Maintenance Schedule ID format", body = HttpResponseObjectEmptyError),
        (status = 401, description = "Unauthorized", body = HttpResponseObjectEmptyError),
        (status = 500, description = "Error deleting Maintenance Schedule", body = HttpResponseObjectEmptyError),
    ),
    security(
        ("Token" = [])
    )
)]
pub async fn delete_maintenance_schedule(
    id: web::Path<String>,
    req: HttpRequest,
    conf: web::Data<Arc<MyCondominiumConfig>>,
) -> HttpResponse {
    let conn = &mut establish_connection_pg(&conf);

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

    let id = match Uuid::parse_str(&id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return HttpResponse::BadRequest().json(HttpResponseObjectEmpty {
                error: true,
                message: "Invalid Resident ID format".to_string(),
            });
        }
    };

    let curr_obj =
        match maintenance_schedule_model::MaintenanceScheduleModel::db_read_by_id(conn, id) {
            Ok(user_req) => user_req,
            Err(e) => {
                return HttpResponse::InternalServerError().json(HttpResponseObjectEmpty {
                    error: true,
                    message: "Error updating Maintenance Schedule".to_string(),
                });
            }
        };

    if !(role.role == UserRoles::Root
        || (role.role == UserRoles::Admin && role.community_id == curr_obj.community_id))
    {
        return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
            error: true,
            message: "Unauthorized".to_string(),
        });
    }

    match maintenance_schedule_model::MaintenanceScheduleModel::db_delete_by_id(conn, id) {
        Ok(_) => HttpResponse::Ok().json(HttpResponseObjectEmpty {
            error: false,
            message: "Maintenance Schedule deleted successfully".to_string(),
        }),
        Err(e) => HttpResponse::Ok().json(HttpResponseObjectEmpty {
            error: true,
            message: format!("Error deleting Maintenance Schedule: {}", e),
        }),
    }
}
