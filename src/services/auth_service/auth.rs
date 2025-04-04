use actix_web::{web, HttpRequest, HttpResponse};
use super::*;
use crate::establish_connection_pg;
use crate::services::{HttpResponseObject, HttpResponseObjectEmpty, HttpResponseObjectEmptyError};

#[utoipa::path(
    post,
    tag = "Authentication",
    path = "/auth",
    responses (
        (status = 200, description = "Authenticated successfully", body = HttpResponseObject<String>),
        (status = 401, description = "Unauthorized", body = HttpResponseObjectEmptyError),
        (status = 500, description = "Error signing in admin", body = HttpResponseObjectEmptyError),
    ),
)]
pub async fn auth(req: HttpRequest) -> HttpResponse {
    let conn = &mut establish_connection_pg();

    match authenticate_user(req) {
        Ok((is_ok, user)) => {
            if is_ok {
                HttpResponse::Ok().json(HttpResponseObject {
                    error: false,
                    message: "Successfully authenticated".to_string(),
                    object: Some(user.user_id),
                })
            } else {
                HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
                    error: true,
                    message: "Unauthorized".to_string(),
                })
            }
        }
        Err(e) => HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
            error: true,
            message: "Unauthorized".to_string(),
        }),
    }

}
