use super::*;
use crate::establish_connection_pg;
use crate::internal::roles::UserRoles;

#[utoipa::path(
    get,
    tag = "Community",
    path = "/list",
    params(
        ("page" = Option<i64>, Query, description = "Page number for pagination (default: 1)"),
        ("per_page" = Option<i64>, Query, description = "Number of items per page for pagination (default: 10)"),
    ),
    responses(
        (status = 200, description = "Got community successfully", body = CommunityListHttpResponse, headers(
            ("X-Total-Pages" = i64, description = "Total number of pages"),
            ("X-Remaining-Pages" = i64, description = "Remaining number of pages")
        )),
        (status = 401, description = "Unauthorized", body = HttpResponseObjectEmptyError),
        (status = 500, description = "Internal server error", body = HttpResponseObjectEmptyError)
    ),
    security(
        ("Token" = [])
    )
)]
pub async fn get_communities(
    query: web::Query<PaginationParams>,
    req: HttpRequest,
) -> HttpResponse {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(10);
    let offset = (page - 1) * per_page;

    let conn = &mut establish_connection_pg();

    match authenticate_user(req.clone(), conn) {
        Ok((role, claims, token)) => {
            if role.role != UserRoles::Root {
                return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
                    error: true,
                    message: "Unauthorized".to_string(),
                });
            }
        }
        Err(_) => {
            return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Unauthorized".to_string(),
            });
        }
    };

    let total_items = match community_model::CommunityModel::table()
        .count()
        .get_result::<i64>(conn)
    {
        Ok(count) => count,
        Err(e) => {
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmpty {
                error: true,
                message: format!("Error getting total items: {}", e),
            });
        }
    };

    match community_model::CommunityModel::table()
        .limit(per_page)
        .offset(offset)
        .load::<community_model::CommunityModel>(conn)
    {
        Ok(res) => {
            let total_pages = (total_items as f64 / per_page as f64).ceil() as i64;
            let remaining_pages = total_pages - page;

            HttpResponse::Ok()
                .insert_header((
                    header::HeaderName::from_static("x-total-pages"),
                    total_pages.to_string(),
                ))
                .insert_header((
                    header::HeaderName::from_static("x-remaining-pages"),
                    remaining_pages.to_string(),
                ))
                .json(HttpResponseObject {
                    error: false,
                    message: "Got communities successfully".to_string(),
                    object: Some(res),
                })
        }
        Err(e) => HttpResponse::InternalServerError().json(HttpResponseObjectEmpty {
            error: true,
            message: format!("Error getting communities: {}", e),
        }),
    }
}

#[utoipa::path(
    get,
    tag = "Community",
    path = "/get/{id}",
    params(
        ("id" = Uuid, Path, description = "Community ID"),
    ),
    responses(
        (status = 200, description = "Got Community successfully", body = CommunityGetHttpResponse),
        (status = 400, description = "Invalid Community ID format or Community ID is required", body = HttpResponseObjectEmptyError),
        (status = 401, description = "Unauthorized", body = HttpResponseObjectEmptyError),
        (status = 500, description = "Internal server error", body = HttpResponseObjectEmptyError)
    ),
    security(
        ("Token" = [])
    )
)]
pub async fn get_community_by_id(id: web::Path<String>, req: HttpRequest) -> HttpResponse {
    let id = id.into_inner();

    let conn = &mut establish_connection_pg();

    let id = match Uuid::parse_str(&id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return HttpResponse::BadRequest().json(HttpResponseObjectEmpty {
                error: true,
                message: "Invalid Community ID format".to_string(),
            });
        }
    };

    match authenticate_user(req.clone(), conn) {
        Ok((role, claims, token)) => {
            if !(role.role == UserRoles::Root
                || (role.role == UserRoles::Admin && role.community_id == Some(id)))
            {
                return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
                    error: true,
                    message: "Unauthorized".to_string(),
                });
            }
        }
        Err(_) => {
            return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Unauthorized".to_string(),
            });
        }
    };

    match community_model::CommunityModel::db_read_by_id(conn, id) {
        Ok(user_req) => HttpResponse::Ok().json(HttpResponseObject {
            error: false,
            message: "Got admin successfully".to_string(),
            object: Some(user_req),
        }),
        Err(e) => HttpResponse::InternalServerError().json(HttpResponseObjectEmpty {
            error: true,
            message: format!("Error getting admin: {}", e),
        }),
    }
}
