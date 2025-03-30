use super::*;
use std::io::ErrorKind;
use crate::utils::{check_password, generate_jwt_token, parse_user_agent};

#[utoipa::path(
    post,
    tag = "Authentication",
    path = "/signin",
    request_body = auth_model::AuthModel,
    responses (
        (status = 200, description = "Signed in successfully", body = HttpResponseObject<String>),
        (status = 401, description = "Unauthorized", body = HttpResponseObjectEmptyError),
        (status = 500, description = "Error signing in admin", body = HttpResponseObjectEmptyError),
    ),
)]
pub async fn sign_in(body: web::Json<auth_model::AuthModel>, req: HttpRequest) -> HttpResponse {
    let conn = &mut establish_connection_pg();
    let email = body.email.trim().to_string();
    let password = body.password.trim().to_string();
    let mut found = false;

    let mut entity_id = match crate::schema::residents::table
        .filter(crate::schema::residents::email.eq(email.clone()))
        .first::<resident_model::ResidentModel>(conn)
        .optional()
    {
        Ok(Some(ent)) => { found = true; Some(ent.id) },
        Ok(None) => None,
        Err(e) => {
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmpty {
                error: true,
                message: format!("Error signing in"),
            });
        }
    };
    
    if !found {
        entity_id = match crate::schema::admins::table
            .filter(crate::schema::admins::email.eq(email.clone()))
            .first::<admin_model::AdminModel>(conn)
            .optional()
        {
            Ok(Some(ent)) => Some(ent.id),
            Ok(None) => None,
            Err(e) => {
                return HttpResponse::InternalServerError().json(HttpResponseObjectEmpty {
                    error: true,
                    message: format!("Error signing in"),
                });
            }
        };
    }
    
    if let Some(id) = entity_id {
        let user_obj = match crate::schema::users::table
            .filter(crate::schema::users::entity_id.eq(id))
            .first::<user_model::UserModel>(conn)
            .optional()
        {
            Ok(Some(ent)) => ent,
            Ok(None) => return HttpResponse::InternalServerError().json(HttpResponseObjectEmpty {
                error: true,
                message: format!("Entity exists but user does not"),
            }),
            Err(e) => {
                return HttpResponse::InternalServerError().json(HttpResponseObjectEmpty {
                    error: true,
                    message: format!("Error signing in"),
                });
            }
        };
        
        match check_password(password, user_obj.password) {
            Ok(val) => if !val {
                return HttpResponse::InternalServerError().json(HttpResponseObjectEmpty {
                    error: true,
                    message: format!("Error signing in"),
                });
            },
            Err(e) => {
                return HttpResponse::InternalServerError().json(HttpResponseObjectEmpty {
                    error: true,
                    message: format!("Error signing in"),
                });
            }
        };

        let token_id = auth_token_model::AuthTokenModel::new_id(conn);

        let token = match generate_jwt_token(user_obj.id, token_id) {
            Ok(new_token) => new_token,
            Err(e) => {
                return HttpResponse::InternalServerError().json(HttpResponseObjectEmpty {
                    error: true,
                    message: format!("Error signing in"),
                });
            }
        };

        let user_agent = match req.headers().get("user-agent") {
            Some(ua) => ua.to_str().unwrap_or("").to_string(),
            None => {
                return HttpResponse::BadRequest().json(HttpResponseObjectEmpty {
                    error: true,
                    message: "User-Agent header is missing".to_string(),
                })
            }
        };

        let ua = match parse_user_agent(user_agent) {
            Ok(ua) => ua,
            Err(_) => {
                return HttpResponse::BadRequest().json(HttpResponseObjectEmpty {
                    error: true,
                    message: "User-Agent header is missing".to_string(),
                })
            }
        };

        let auth_token = auth_token_model::AuthTokenModel {
            user_id: user_obj.id,
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
            Err(e) => HttpResponse::InternalServerError().json(HttpResponseObjectEmptyError {
                error: true,
                message: format!("Internal server error"),
            }),
        }
    } else {
        HttpResponse::InternalServerError().json(HttpResponseObjectEmpty {
            error: true,
            message: format!("Error signing in"),
        })
    }
}
