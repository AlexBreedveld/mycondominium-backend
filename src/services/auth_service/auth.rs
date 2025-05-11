use super::*;
use crate::establish_connection_pg;
use crate::models::user_model::UserModelResult;
use crate::services::{HttpResponseObject, HttpResponseObjectEmpty, HttpResponseObjectEmptyError};
use actix_web::{HttpRequest, HttpResponse, web};

#[utoipa::path(
    get,
    description = "Validates user's token and returns user's profile information based on their role.",
    tag = "Authentication",
    path = "/auth",
    responses (
        (status = 200, description = "Authenticated successfully", body = HttpResponseObject<auth_model::AuthUserModelResult>),
        (status = 401, description = "Unauthorized", body = HttpResponseObjectEmptyError),
        (status = 500, description = "Error signing in admin", body = HttpResponseObjectEmptyError),
    ),
    security(
        ("Token" = [])
    )
)]
pub async fn auth(req: HttpRequest, conf: web::Data<Arc<MyCondominiumConfig>>) -> HttpResponse {
    let conn = &mut establish_connection_pg(&conf);

    match authenticate_user(req, conn, conf) {
        Ok((role, claims, token)) => {
            let user = match user_model::UserModel::db_read_by_id(conn, role.user_id) {
                Ok(user) => user,
                Err(e) => {
                    return HttpResponse::InternalServerError().json(
                        HttpResponseObjectEmptyError {
                            error: true,
                            message: "Error getting user".to_string(),
                        },
                    );
                }
            };

            if role.role == UserRoles::Admin || role.role == UserRoles::Root {
                let admin = match admin_model::AdminModel::db_read_by_id(conn, user.entity_id) {
                    Ok(admin) => admin,
                    Err(e) => {
                        return HttpResponse::InternalServerError().json(
                            HttpResponseObjectEmptyError {
                                error: true,
                                message: "Error getting user".to_string(),
                            },
                        );
                    }
                };

                HttpResponse::Ok().json(HttpResponseObject {
                    error: false,
                    message: "Successfully authenticated".to_string(),
                    object: Some(auth_model::AuthUserModelResult {
                        admin: Some(admin),
                        resident: None,
                        user: UserModelResult {
                            id: user.id,
                            entity_id: user.entity_id,
                            entity_type: user.entity_type,
                            admin_id: user.admin_id,
                            resident_id: user.resident_id,
                            created_at: user.created_at,
                            updated_at: user.updated_at,
                        },
                        role,
                    }),
                })
            } else if role.role == UserRoles::Resident {
                let resident =
                    match resident_model::ResidentModel::db_read_by_id(conn, user.entity_id) {
                        Ok(resident) => resident,
                        Err(e) => {
                            return HttpResponse::InternalServerError().json(
                                HttpResponseObjectEmptyError {
                                    error: true,
                                    message: "Error getting user".to_string(),
                                },
                            );
                        }
                    };

                HttpResponse::Ok().json(HttpResponseObject {
                    error: false,
                    message: "Successfully authenticated".to_string(),
                    object: Some(auth_model::AuthUserModelResult {
                        admin: None,
                        resident: Some(resident),
                        user: UserModelResult {
                            id: user.id,
                            entity_id: user.entity_id,
                            entity_type: user.entity_type,
                            admin_id: user.admin_id,
                            resident_id: user.resident_id,
                            created_at: user.created_at,
                            updated_at: user.updated_at,
                        },
                        role,
                    }),
                })
            } else {
                return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
                    error: true,
                    message: "Unauthorized".to_string(),
                });
            }
        }
        Err(e) => HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
            error: true,
            message: "Unauthorized".to_string(),
        }),
    }
}

#[utoipa::path(
    get,
    tag = "Authentication",
    path = "/signout",
    responses (
        (status = 200, description = "Signed out successfully", body = HttpResponseObject<String>),
        (status = 401, description = "Unauthorized", body = HttpResponseObjectEmptyError),
        (status = 500, description = "Error signing out", body = HttpResponseObjectEmptyError),
    ),
    security(
        ("Token" = [])
    )
)]
pub async fn sign_out(req: HttpRequest, conf: web::Data<Arc<MyCondominiumConfig>>) -> HttpResponse {
    let conn = &mut establish_connection_pg(&conf);

    let (role, claims, token) = match authenticate_user(req, conn, conf) {
        Ok((role, claims, token)) => (role, claims, token),
        Err(e) => {
            return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Unauthorized".to_string(),
            });
        }
    };

    match auth_token_model::AuthTokenModel::db_delete_by_id(conn, token.id) {
        Ok(_) => HttpResponse::Ok().json(HttpResponseObject {
            error: false,
            message: "Successfully signed out".to_string(),
            object: Some(String::from("Signed out successfully")),
        }),
        Err(e) => HttpResponse::InternalServerError().json(HttpResponseObjectEmptyError {
            error: true,
            message: "Error signing out".to_string(),
        }),
    }
}
