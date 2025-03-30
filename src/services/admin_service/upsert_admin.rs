use super::*;
use crate::models::admin_model::AdminModel;
use crate::utils::hash_password;
use std::io::ErrorKind;

#[utoipa::path(
    post,
    tag = "Admin",
    path = "/new",
    request_body = admin_model::AdminModelNew,
    responses (
        (status = 200, description = "Admin added successfully", body = HttpResponseObjectEmptyEntity),
        (status = 400, description = "Error adding admin", body = HttpResponseObjectEmptyError),
        (status = 401, description = "Unauthorized", body = HttpResponseObjectEmptyError),
        (status = 500, description = "Error adding admin", body = HttpResponseObjectEmptyError),
    ),
)]
pub async fn new_admin(body: web::Json<admin_model::AdminModelNew>) -> HttpResponse {
    let conn = &mut establish_connection_pg();

    let body = body.into_inner();

    if let Err(validation_errors) = body.validate() {
        return HttpResponse::BadRequest().json(validation_errors);
    }

    let new_obj = admin_model::AdminModel {
        id: admin_model::AdminModel::new_id_user(conn),
        first_name: body.first_name,
        last_name: body.last_name,
        phone: body.phone,
        email: body.email,
        created_at: chrono::Utc::now().naive_utc(),
        updated_at: chrono::Utc::now().naive_utc(),
    };

    match new_obj.db_insert(conn) {
        Ok(_) => (),
        Err(e) => {
            log::error!("Error creating admin: {}", e);
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Error creating admin".to_string(),
            });
        }
    };

    let hashed_password = match hash_password(body.password) {
        Ok(passwd) => passwd,
        Err(e) => {
            log::error!("Error creating admin: {}", e);
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Error creating admin".to_string(),
            });
        }
    };

    let new_obj_user = user_model::UserModel {
        id: user_model::UserModel::new_id(conn),
        entity_id: new_obj.id,
        entity_type: UserTypes::Admin,
        admin_id: Some(new_obj.id),
        resident_id: None,
        password: hashed_password,
        created_at: chrono::Utc::now().naive_utc(),
        updated_at: chrono::Utc::now().naive_utc(),
    };

    match new_obj_user.db_insert(conn) {
        Ok(_) => (),
        Err(e) => {
            log::error!("Error creating admin: {}", e);
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Error creating admin".to_string(),
            });
        }
    };

    let new_obj_user_role = user_role_model::UserRoleModel {
        id: user_role_model::UserRoleModel::new_id(conn),
        user_id: new_obj_user.id,
        role: body.role,
        community_id: body.community_id,
        created_at: chrono::Utc::now().naive_utc(),
        updated_at: chrono::Utc::now().naive_utc(),
    };

    match new_obj_user_role.db_insert(conn) {
        Ok(_) => (),
        Err(e) => {
            log::error!("Error creating admin: {}", e);
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Error creating admin".to_string(),
            });
        }
    };

    HttpResponse::Ok().json(HttpResponseObjectEmptyEntity {
        error: false,
        message: "Admin created successfully".to_string(),
        entity_id: Some(new_obj.id),
    })
}

#[utoipa::path(
    put,
    tag = "Admin",
    path = "/update/{id}",
    request_body = admin_model::AdminModelNew,
    params(
        ("id" = Uuid, Path, description = "Admin ID"),
    ),
    responses (
        (status = 200, description = "Admin updated successfully", body = HttpResponseObjectEmpty),
        (status = 400, description = "Error updating Admin", body = HttpResponseObjectEmptyError),
        (status = 401, description = "Unauthorized", body = HttpResponseObjectEmptyError),
        (status = 500, description = "Internal server error", body = HttpResponseObjectEmptyError),
    ),
)]
pub async fn update_admin(
    id: web::Path<String>,
    body: web::Json<admin_model::AdminModelNew>,
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
                message: "Invalid Admin ID format".to_string(),
            });
        }
    };

    let curr_obj = match admin_model::AdminModel::db_read_by_id(conn, id) {
        Ok(ent_req) => ent_req,
        Err(e) => {
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmpty {
                error: true,
                message: format!("Error getting admin: {}", e),
            });
        }
    };

    if !check_email_valid(conn, body.email.clone(), curr_obj.email).unwrap() {
        return HttpResponse::BadRequest().json(HttpResponseObjectEmpty {
            error: true,
            message: "Email already exists".to_string(),
        });
    }

    let new_obj = admin_model::AdminModel {
        id: curr_obj.id,
        first_name: body.first_name,
        last_name: body.last_name,
        phone: body.phone,
        email: body.email,
        created_at: curr_obj.created_at,
        updated_at: chrono::Utc::now().naive_utc(),
    };

    match new_obj.db_update(conn) {
        Ok(_) => HttpResponse::Ok().json(HttpResponseObjectEmpty {
            error: false,
            message: "Admin updated successfully".to_string(),
        }),
        Err(e) => HttpResponse::Ok().json(HttpResponseObjectEmpty {
            error: true,
            message: format!("Error creating admin: {}", e),
        }),
    }
}

#[utoipa::path(
    delete,
    tag = "Admin",
    path = "/delete/{id}",
    params(
        ("id" = Uuid, Path, description = "Admin ID"),
    ),
    responses (
        (status = 200, description = "Admin deleted successfully", body = HttpResponseObjectEmpty),
        (status = 400, description = "Invalid Admin ID format", body = HttpResponseObjectEmptyError),
        (status = 401, description = "Unauthorized", body = HttpResponseObjectEmptyError),
        (status = 500, description = "Error deleting Admin", body = HttpResponseObjectEmptyError),
    ),
)]
pub async fn delete_admin(id: web::Path<String>) -> HttpResponse {
    let conn = &mut establish_connection_pg();

    let id = match Uuid::parse_str(&id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return HttpResponse::BadRequest().json(HttpResponseObjectEmpty {
                error: true,
                message: "Invalid Admin ID format".to_string(),
            });
        }
    };

    match admin_model::AdminModel::db_delete_by_id(conn, id) {
        Ok(_) => HttpResponse::Ok().json(HttpResponseObjectEmpty {
            error: false,
            message: "Admin deleted successfully".to_string(),
        }),
        Err(e) => HttpResponse::InternalServerError().json(HttpResponseObjectEmptyError {
            error: true,
            message: format!("Error deleting admin: {}", e),
        }),
    }
}

fn check_email_valid(
    conn: &mut PgConnection,
    req_email: String,
    curr_email: String,
) -> Result<bool, std::io::Error> {
    if req_email.trim() == curr_email.trim() {
        return Ok(true);
    }

    match crate::schema::admins::table
        .filter(crate::schema::admins::email.eq(req_email.clone()))
        .count()
        .get_result::<i64>(conn)
    {
        Ok(num) => {
            if num != 0 {
                return Ok(false);
            }
        }
        Err(e) => {
            return Err(std::io::Error::new(
                ErrorKind::Other,
                "Error checking if email exists",
            ));
        }
    };

    match crate::schema::residents::table
        .filter(crate::schema::residents::email.eq(req_email.clone()))
        .count()
        .get_result::<i64>(conn)
    {
        Ok(num) => {
            if num != 0 {
                return Ok(false);
            }
        }
        Err(e) => {
            return Err(std::io::Error::new(
                ErrorKind::Other,
                "Error checking if email exists",
            ));
        }
    };

    Ok(true)
}
