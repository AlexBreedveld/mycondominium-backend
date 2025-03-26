use super::*;
use crate::establish_connection_pg;

#[utoipa::path(
    get,
    tag = "Admin",
    path = "/list",
    params(
        ("page" = Option<i64>, Query, description = "Page number for pagination (default: 1)"),
        ("per_page" = Option<i64>, Query, description = "Number of items per page for pagination (default: 10)"),
    ),
    responses(
        (status = 200, description = "Got admins successfully", body = AdminListHttpResponse, headers(
            ("X-Total-Pages" = i64, description = "Total number of pages"),
            ("X-Remaining-Pages" = i64, description = "Remaining number of pages")
        )),
        (status = 401, description = "Unauthorized", body = HttpResponseObjectEmptyError),
        (status = 500, description = "Internal server error", body = HttpResponseObjectEmptyError)
    ),
)]
pub async fn get_admins(query: web::Query<PaginationParams>) -> HttpResponse {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(10);
    let offset = (page - 1) * per_page;

    let conn = &mut establish_connection_pg();

    let total_items = match admin_model::AdminModel::db_count_all(conn) {
        Ok(count) => count,
        Err(e) => {
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmpty {
                error: true,
                message: format!("Error getting total items: {}", e),
            });
        }
    };

    match Vec::<admin_model::AdminModel>::db_read_by_range(conn, per_page, offset) {
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
                    message: "Got admins successfully".to_string(),
                    object: Some(res),
                })
        }
        Err(e) => HttpResponse::InternalServerError().json(HttpResponseObjectEmpty {
            error: true,
            message: format!("Error getting admins: {}", e),
        }),
    }
}

#[utoipa::path(
    get,
    tag = "Admin",
    path = "/get/{id}",
    params(
        ("id" = Uuid, Path, description = "Admin ID"),
    ),
    responses(
        (status = 200, description = "Got admin successfully", body = AdminGetHttpResponse),
        (status = 400, description = "Invalid Admin ID format or Admin ID is required", body = HttpResponseObjectEmptyError),
        (status = 401, description = "Unauthorized", body = HttpResponseObjectEmptyError),
        (status = 500, description = "Internal server error", body = HttpResponseObjectEmptyError)
    ),
)]
pub async fn get_admin_by_id(id: web::Path<String>) -> HttpResponse {
    let id = id.into_inner();

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

    match admin_model::AdminModel::db_read_by_id(conn, id) {
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
