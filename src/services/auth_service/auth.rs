use super::*;
use crate::establish_connection_pg;
use crate::services::{HttpResponseObject, HttpResponseObjectEmpty, HttpResponseObjectEmptyError};
use actix_web::{HttpRequest, HttpResponse, web};

#[utoipa::path(
    get,
    tag = "Authentication",
    path = "/auth",
    responses (
        (status = 200, description = "Authenticated successfully", body = HttpResponseObject<String>),
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
        Ok((role, claims, token)) => HttpResponse::Ok().json(HttpResponseObject {
            error: false,
            message: "Successfully authenticated".to_string(),
            object: Some(role.user_id),
        }),
        Err(e) => HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
            error: true,
            message: "Unauthorized".to_string(),
        }),
    }
}

#[utoipa::path(
    get,
    tag = "Authentication",
    path = "/get_user",
    responses (
        (status = 200, description = "Got User successfully", body = HttpResponseObject<user_model::UserModel>),
        (status = 401, description = "Unauthorized", body = HttpResponseObjectEmptyError),
        (status = 500, description = "Error signing in admin", body = HttpResponseObjectEmptyError),
    ),
    security(
        ("Token" = [])
    )
)]
pub async fn auth_get_user(
    req: HttpRequest,
    conf: web::Data<Arc<MyCondominiumConfig>>,
) -> HttpResponse {
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

            HttpResponse::Ok().json(HttpResponseObject {
                error: false,
                message: "Successfully authenticated".to_string(),
                object: Some(user),
            })
        }
        Err(e) => HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
            error: true,
            message: "Unauthorized".to_string(),
        }),
    }
}
