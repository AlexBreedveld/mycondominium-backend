use super::*;
use crate::utilities::auth_utils::{check_password, parse_user_agent};
use std::time::Duration;
use tokio::time::sleep;

#[utoipa::path(
    post,
    tag = "Authentication",
    path = "/signin",
    request_body = auth_model::AuthModel,
    responses (
        (status = 200, description = "Signed in successfully", body = HttpResponseObject<String>),
        (status = 401, description = "Unauthorized", body = HttpResponseObjectEmptyError),
        (status = 500, description = "Error signing in", body = HttpResponseObjectEmptyError),
    ),
)]
pub async fn sign_in(
    body: web::Json<auth_model::AuthModel>,
    req: HttpRequest,
    conf: web::Data<Arc<MyCondominiumConfig>>,
) -> HttpResponse {
    let conn = &mut establish_connection_pg(&conf);
    let email = body.email.trim().to_string();
    let password = body.password.trim().to_string();

    sleep(Duration::from_secs(3)).await;

    let user = match UserModel::find_user_by_email(conn, email) {
        Ok(usr) => usr,
        Err(e) => {
            log::error!("Error getting user: {}", e);
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmpty {
                error: true,
                message: "Error signing in".to_string(),
            });
        }
    };

    match check_password(password, user.user.password) {
        Ok(val) => {
            if !val {
                return HttpResponse::InternalServerError().json(HttpResponseObjectEmpty {
                    error: true,
                    message: "Error signing in".to_string(),
                });
            }
        }
        Err(e) => {
            log::error!("Error checking password: {}", e);
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmpty {
                error: true,
                message: "Error signing in".to_string(),
            });
        }
    };

    let token_id = auth_token_model::AuthTokenModel::new_id(conn);

    let token = match generate_jwt_token_no_env(
        user.user.id,
        token_id,
        conf.auth.token_secret_key.clone(),
        conf.auth.token_expiration_days as i64,
    ) {
        Ok(new_token) => new_token,
        Err(e) => {
            log::error!("Error generating JWT token: {}", e);
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmpty {
                error: true,
                message: "Error signing in".to_string(),
            });
        }
    };

    let user_agent = match req.headers().get("user-agent") {
        Some(ua) => ua.to_str().unwrap_or("").to_string(),
        None => {
            return HttpResponse::BadRequest().json(HttpResponseObjectEmpty {
                error: true,
                message: "User-Agent header is missing".to_string(),
            });
        }
    };

    let ua = match parse_user_agent(user_agent) {
        Ok(ua) => ua,
        Err(_) => {
            return HttpResponse::BadRequest().json(HttpResponseObjectEmpty {
                error: true,
                message: "User-Agent header is missing".to_string(),
            });
        }
    };

    let auth_token = auth_token_model::AuthTokenModel {
        user_id: user.user.id,
        id: token_id,
        time_added: chrono::Utc::now().naive_utc(),
        active: true,
        time_last_used: chrono::Utc::now().naive_utc(),
        device: ua.device.name,
        browser: ua.product.name,
        version: ua.product.major,
        cpu_arch: ua.cpu.architecture,
    };

    match auth_token.db_insert(conn) {
        Ok(_) => HttpResponse::Ok().json(HttpResponseObject {
            error: false,
            message: "Successfully authenticated".to_string(),
            object: Some(token),
        }),
        Err(e) => {
            log::error!("Error inserting auth token: {}", e);
            HttpResponse::InternalServerError().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Internal server error".to_string(),
            })
        }
    }
}
