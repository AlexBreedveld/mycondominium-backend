use super::*;

#[utoipa::path(
    get,
    tag = "Election",
    path = "/vote/{id}",
    params(
        ("id" = Uuid, Path, description = "Election ID"),
    ),
    responses(
        (status = 200, description = "Got Election successfully", body = HttpResponseObject<bool>),
        (status = 400, description = "Invalid Election ID format or Election ID is required", body = HttpResponseObjectEmptyError),
        (status = 401, description = "Unauthorized", body = HttpResponseObjectEmptyError),
        (status = 500, description = "Internal server error", body = HttpResponseObjectEmptyError)
    ),
    security(
        ("Token" = [])
    )
)]
pub async fn get_can_vote(
    id: web::Path<String>,
    req: HttpRequest,
    conf: web::Data<Arc<MyCondominiumConfig>>,
) -> HttpResponse {
    let id = id.into_inner();

    let conn = &mut establish_connection_pg(&conf);

    let id = match Uuid::parse_str(&id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return HttpResponse::BadRequest().json(HttpResponseObjectEmpty {
                error: true,
                message: "Invalid Election ID format".to_string(),
            });
        }
    };

    let (role, _claims, _token) = match authenticate_user(req.clone(), conn, conf) {
        Ok((role, claims, token)) => (role, claims, token),
        Err(_) => {
            return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Unauthorized".to_string(),
            });
        }
    };

    match election_model::ElectionModel::can_user_vote(role, conn, id) {
        Ok(user_req) => HttpResponse::Ok().json(HttpResponseObject {
            error: false,
            message: "Got Election vote successfully".to_string(),
            object: Some(user_req),
        }),
        Err(e) => {
            log::error!("Error getting Election vote: {}", e);
            HttpResponse::InternalServerError().json(HttpResponseObjectEmpty {
                error: true,
                message: format!("Error getting Election vote: {}", e),
            })
        }
    }
}
