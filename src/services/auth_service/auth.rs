use super::*;
use crate::establish_connection_pg;
use crate::services::{HttpResponseObject, HttpResponseObjectEmpty, HttpResponseObjectEmptyError};
use actix_web::{HttpRequest, HttpResponse, web};

#[utoipa::path(
    post,
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

    match authenticate_user(req, conn) {
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
