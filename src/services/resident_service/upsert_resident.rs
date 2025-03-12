use super::*;

#[utoipa::path(
    post,
    tag = "Resident",
    path = "/new",
    request_body = resident_model::ResidentModelNew,
    responses (
        (status = 200, description = "Resident added successfully", body = HttpResponseObjectEmptyEntity),
        (status = 400, description = "Error adding resident", body = HttpResponseObjectEmptyError),
        (status = 401, description = "Unauthorized", body = HttpResponseObjectEmptyError),
        (status = 500, description = "Error adding resident", body = HttpResponseObjectEmptyError),
    ),
)]
pub async fn new_resident (body: web::Json<resident_model::ResidentModelNew>) -> HttpResponse {
    let conn = &mut establish_connection_pg();

    let body = body.into_inner();

    if let Err(validation_errors) = body.validate() {
        return HttpResponse::BadRequest().json(validation_errors);
    }

    let new_obj = resident_model::ResidentModel {
        id: Uuid::new_v4(),
        first_name: body.first_name,
        last_name: body.last_name,
        unit_number: body.unit_number,
        address: body.address,
        phone: body.phone,
        email: body.email,
        date_of_birth: body.date_of_birth,
        resident_since: body.resident_since,
        is_active: body.is_active,
        created_at: chrono::Utc::now().naive_utc(),
        updated_at: chrono::Utc::now().naive_utc(),
    };

    match new_obj.db_insert(conn) {
        Ok(_) => HttpResponse::Ok().json(HttpResponseObjectEmptyEntity {
            error: false,
            message: "User created successfully".to_string(),
            entity_id: Some(new_obj.id),
        }),
        Err(e) => HttpResponse::InternalServerError().json(HttpResponseObjectEmptyError {
            error: true,
            message: format!("Error creating user: {}", e),
        }),
    }
}

#[utoipa::path(
    put,
    tag = "Resident",
    path = "/update/{id}",
    request_body = resident_model::ResidentModelNew,
    params(
        ("id" = Uuid, Path, description = "Resident ID"),
    ),
    responses (
        (status = 200, description = "Resident updated successfully", body = HttpResponseObjectEmpty),
        (status = 400, description = "Error updating Resident", body = HttpResponseObjectEmptyError),
        (status = 401, description = "Unauthorized", body = HttpResponseObjectEmptyError),
        (status = 500, description = "Internal server error", body = HttpResponseObjectEmptyError),
    ),
)]
pub async fn update_resident(
    id: web::Path<String>,
    body: web::Json<resident_model::ResidentModelNew>
) -> HttpResponse {
    let conn = &mut establish_connection_pg();
    let body = body.into_inner();

    if let Err(validation_errors) = body.validate() {
        return HttpResponse::BadRequest().json(validation_errors);
    }

    let id = match Uuid::parse_str(&id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return HttpResponse::BadRequest().json(HttpResponseObjectEmpty {
                error: true,
                message: "Invalid Resident ID format".to_string(),
            })
        }
    };

    let curr_obj = match resident_model::ResidentModel::db_read_by_id(conn, id) {
        Ok(user_req) => user_req,
        Err(e) => {
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmpty {
                error: true,
                message: format!("Error getting resident: {}", e),
            })
        }
    };

    let new_obj = resident_model::ResidentModel {
        id: curr_obj.id,
        first_name: body.first_name,
        last_name: body.last_name,
        unit_number: body.unit_number,
        address: body.address,
        phone: body.phone,
        email: body.email,
        date_of_birth: body.date_of_birth,
        resident_since: body.resident_since,
        is_active: body.is_active,
        created_at: curr_obj.created_at,
        updated_at: chrono::Utc::now().naive_utc(),
    };

    match new_obj.db_update(conn) {
        Ok(_) => HttpResponse::Ok().json(HttpResponseObjectEmpty {
            error: false,
            message: "Resident updated successfully".to_string(),
        }),
        Err(e) => HttpResponse::Ok().json(HttpResponseObjectEmpty {
            error: true,
            message: format!("Error creating resident: {}", e),
        }),
    }
}

#[utoipa::path(
    delete,
    tag = "Resident",
    path = "/delete/{id}",
    params(
        ("id" = Uuid, Path, description = "Resident ID"),
    ),
    responses (
        (status = 200, description = "Resident deleted successfully", body = HttpResponseObjectEmpty),
        (status = 400, description = "Invalid Resident ID format", body = HttpResponseObjectEmptyError),
        (status = 401, description = "Unauthorized", body = HttpResponseObjectEmptyError),
        (status = 500, description = "Error deleting Resident", body = HttpResponseObjectEmptyError),
    ),
)]
pub async fn delete_resident(
    id: web::Path<String>,
) -> HttpResponse {
    let conn = &mut establish_connection_pg();

    let id = match Uuid::parse_str(&id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return HttpResponse::BadRequest().json(HttpResponseObjectEmpty {
                error: true,
                message: "Invalid Resident ID format".to_string(),
            })
        }
    };

    match resident_model::ResidentModel::db_delete_by_id(conn, id) {
        Ok(_) => HttpResponse::Ok().json(HttpResponseObjectEmpty {
            error: false,
            message: "Resident deleted successfully".to_string(),
        }),
        Err(e) => HttpResponse::InternalServerError().json(HttpResponseObjectEmptyError {
            error: true,
            message: format!("Error deleting resident: {}", e),
        }),
    }
}
