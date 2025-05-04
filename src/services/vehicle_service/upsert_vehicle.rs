use super::*;

#[utoipa::path(
    post,
    tag = "Vehicle",
    path = "/new",
    request_body = vehicle_model::VehicleModelNew,
    responses (
        (status = 200, description = "Vehicle added successfully", body = HttpResponseObjectEmptyEntity),
        (status = 400, description = "Error adding vehicle", body = HttpResponseObjectEmptyError),
        (status = 401, description = "Unauthorized", body = HttpResponseObjectEmptyError),
        (status = 500, description = "Error adding vehicle", body = HttpResponseObjectEmptyError),
    ),
    security(
        ("Token" = [])
    )
)]
pub async fn new_vehicle(
    body: web::Json<vehicle_model::VehicleModelNew>,
    req: HttpRequest,
    conf: web::Data<Arc<MyCondominiumConfig>>,
) -> HttpResponse {
    let conn = &mut establish_connection_pg(&conf);
    let body = body.into_inner();

    let (role, claims, token) = match authenticate_user(req.clone(), conn, conf) {
        Ok((role, claims, token)) => (role, claims, token),
        Err(_) => {
            return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Unauthorized".to_string(),
            });
        }
    };

    if let Err(validation_errors) = body.validate() {
        return HttpResponse::BadRequest().json(validation_errors);
    }

    // Check if the resident belongs to the same community as the admin
    if role.role == UserRoles::Admin {
        // Fetch the resident's role and community ID
        use crate::schema::{residents, user_roles, users};
        use diesel::prelude::*;

        let resident_community_id = match users::table
            .inner_join(residents::table.on(users::entity_id.eq(residents::id)))
            .inner_join(user_roles::table.on(user_roles::user_id.eq(users::id)))
            .filter(residents::id.eq(body.resident_id))
            .select(user_roles::community_id)
            .first::<Option<Uuid>>(conn)
        {
            Ok(community_id) => community_id,
            Err(e) => {
                return HttpResponse::InternalServerError().json(HttpResponseObjectEmptyError {
                    error: true,
                    message: format!("Error checking resident's community: {}", e),
                });
            }
        };

        if resident_community_id != role.community_id {
            return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Unauthorized: Resident not in the same community".to_string(),
            });
        }
    } else if role.role != UserRoles::Root {
        return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
            error: true,
            message: "Unauthorized".to_string(),
        });
    }

    let new_vehicle = vehicle_model::VehicleModel {
        id: Uuid::new_v4(),
        resident_id: body.resident_id,
        license_plate: body.license_plate,
        model: body.model,
        color: body.color,
        created_at: chrono::Utc::now().naive_utc(),
        updated_at: chrono::Utc::now().naive_utc(),
    };

    match new_vehicle.db_insert(conn) {
        Ok(_) => HttpResponse::Ok().json(HttpResponseObjectEmptyEntity {
            error: false,
            message: "Vehicle created successfully".to_string(),
            entity_id: Some(new_vehicle.id),
        }),
        Err(e) => HttpResponse::InternalServerError().json(HttpResponseObjectEmptyError {
            error: true,
            message: format!("Error creating vehicle: {}", e),
        }),
    }
}

#[utoipa::path(
    put,
    tag = "Vehicle",
    path = "/update/{id}",
    params(
        ("id" = Uuid, Path, description = "Vehicle ID"),
    ),
    request_body = vehicle_model::VehicleModelNew,
    responses (
        (status = 200, description = "Vehicle updated successfully", body = HttpResponseObjectEmpty),
        (status = 400, description = "Error updating vehicle", body = HttpResponseObjectEmptyError),
        (status = 401, description = "Unauthorized", body = HttpResponseObjectEmptyError),
        (status = 500, description = "Internal server error", body = HttpResponseObjectEmptyError),
    ),
    security(
        ("Token" = [])
    )
)]
pub async fn update_vehicle(
    id: web::Path<String>,
    body: web::Json<vehicle_model::VehicleModelNew>,
    req: HttpRequest,
    conf: web::Data<Arc<MyCondominiumConfig>>,
) -> HttpResponse {
    let conn = &mut establish_connection_pg(&conf);
    let id = id.into_inner();
    let body = body.into_inner();

    let (role, claims, token) = match authenticate_user(req.clone(), conn, conf) {
        Ok((role, claims, token)) => (role, claims, token),
        Err(_) => {
            return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Unauthorized".to_string(),
            });
        }
    };

    if let Err(validation_errors) = body.validate() {
        return HttpResponse::BadRequest().json(validation_errors);
    }

    let id = match Uuid::parse_str(&id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return HttpResponse::BadRequest().json(HttpResponseObjectEmpty {
                error: true,
                message: "Invalid Vehicle ID format".to_string(),
            });
        }
    };

    // Check if the resident belongs to the same community as the admin
    if role.role == UserRoles::Admin {
        // Fetch the resident's role and community ID
        use crate::schema::{residents, user_roles, users};
        use diesel::prelude::*;

        let resident_community_id = match users::table
            .inner_join(residents::table.on(users::entity_id.eq(residents::id)))
            .inner_join(user_roles::table.on(user_roles::user_id.eq(users::id)))
            .filter(residents::id.eq(body.resident_id))
            .select(user_roles::community_id)
            .first::<Option<Uuid>>(conn)
        {
            Ok(community_id) => community_id,
            Err(e) => {
                return HttpResponse::InternalServerError().json(HttpResponseObjectEmptyError {
                    error: true,
                    message: format!("Error checking resident's community: {}", e),
                });
            }
        };

        if resident_community_id != role.community_id {
            return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Unauthorized: Resident not in the same community".to_string(),
            });
        }
    } else if role.role != UserRoles::Root {
        return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
            error: true,
            message: "Unauthorized".to_string(),
        });
    }

    let existing_vehicle = match vehicle_model::VehicleModel::db_read_by_id(conn, id) {
        Ok(vehicle) => vehicle,
        Err(e) => {
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmpty {
                error: true,
                message: format!("Error getting vehicle: {}", e),
            });
        }
    };

    let updated_vehicle = vehicle_model::VehicleModel {
        id: existing_vehicle.id,
        resident_id: body.resident_id,
        license_plate: body.license_plate,
        model: body.model,
        color: body.color,
        created_at: existing_vehicle.created_at,
        updated_at: chrono::Utc::now().naive_utc(),
    };

    match updated_vehicle.db_update(conn) {
        Ok(_) => HttpResponse::Ok().json(HttpResponseObjectEmpty {
            error: false,
            message: "Vehicle updated successfully".to_string(),
        }),
        Err(e) => HttpResponse::InternalServerError().json(HttpResponseObjectEmptyError {
            error: true,
            message: format!("Error updating vehicle: {}", e),
        }),
    }
}

#[utoipa::path(
    delete,
    tag = "Vehicle",
    path = "/delete/{id}",
    params(
        ("id" = Uuid, Path, description = "Vehicle ID"),
    ),
    responses (
        (status = 200, description = "Vehicle deleted successfully", body = HttpResponseObjectEmpty),
        (status = 400, description = "Invalid Vehicle ID format", body = HttpResponseObjectEmptyError),
        (status = 401, description = "Unauthorized", body = HttpResponseObjectEmptyError),
        (status = 500, description = "Error deleting vehicle", body = HttpResponseObjectEmptyError),
    ),
    security(
        ("Token" = [])
    )
)]
pub async fn delete_vehicle(
    id: web::Path<String>,
    req: HttpRequest,
    conf: web::Data<Arc<MyCondominiumConfig>>,
) -> HttpResponse {
    let conn = &mut establish_connection_pg(&conf);
    let id = id.into_inner();

    let (role, claims, token) = match authenticate_user(req.clone(), conn, conf) {
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
                message: "Invalid Vehicle ID format".to_string(),
            });
        }
    };

    // Additional access control for delete
    let vehicle = match vehicle_model::VehicleModel::db_read_by_id(conn, id) {
        Ok(vehicle) => vehicle,
        Err(e) => {
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmpty {
                error: true,
                message: format!("Error getting vehicle: {}", e),
            });
        }
    };

    // Check if the resident belongs to the same community as the admin
    if role.role == UserRoles::Admin {
        // Fetch the resident's role and community ID
        use crate::schema::{residents, user_roles, users};
        use diesel::prelude::*;

        let resident_community_id = match users::table
            .inner_join(residents::table.on(users::entity_id.eq(residents::id)))
            .inner_join(user_roles::table.on(user_roles::user_id.eq(users::id)))
            .filter(residents::id.eq(vehicle.resident_id))
            .select(user_roles::community_id)
            .first::<Option<Uuid>>(conn)
        {
            Ok(community_id) => community_id,
            Err(e) => {
                return HttpResponse::InternalServerError().json(HttpResponseObjectEmptyError {
                    error: true,
                    message: format!("Error checking resident's community: {}", e),
                });
            }
        };

        if resident_community_id != role.community_id {
            return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Unauthorized: Resident not in the same community".to_string(),
            });
        }
    } else if role.role != UserRoles::Root {
        return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
            error: true,
            message: "Unauthorized".to_string(),
        });
    }

    match vehicle_model::VehicleModel::db_delete_by_id(conn, id) {
        Ok(_) => HttpResponse::Ok().json(HttpResponseObjectEmpty {
            error: false,
            message: "Vehicle deleted successfully".to_string(),
        }),
        Err(e) => HttpResponse::InternalServerError().json(HttpResponseObjectEmptyError {
            error: true,
            message: format!("Error deleting vehicle: {}", e),
        }),
    }
}
