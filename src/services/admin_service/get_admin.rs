use super::*;
use crate::establish_connection_pg;
use crate::internal::roles::UserRoles;

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
    security(
        ("Token" = [])
    )
)]
pub async fn get_admins(query: web::Query<PaginationParams>, req: HttpRequest, conf: web::Data<Arc<MyCondominiumConfig>>) -> HttpResponse {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(10);
    let offset = (page - 1) * per_page;

    let conn = &mut establish_connection_pg(&conf);

    let admin_role = match authenticate_user(req.clone(), conn, conf) {
        Ok((role, claims, token)) => {
            if role.role == UserRoles::Root || role.role == UserRoles::Admin {
                role
            } else {
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

    let mut total_items_query = user_role_model::UserRoleModel::table()
        .inner_join(users::table.on(users::id.eq(user_roles::user_id)))
        .inner_join(admins::table.on(admins::id.eq(users::entity_id)))
        .filter(users::entity_type.eq("admin"))
        .into_boxed();

    match admin_role.role {
        UserRoles::Root => {
            total_items_query = total_items_query.filter(
                user_roles::role
                    .eq(UserRoles::Root)
                    .or(user_roles::role.eq(UserRoles::Admin)),
            );
        }
        UserRoles::Admin => {
            total_items_query = total_items_query.filter(
                user_roles::role
                    .eq(UserRoles::Admin)
                    .and(user_roles::community_id.eq(admin_role.community_id)),
            );
        }
        _ => return HttpResponse::InternalServerError().json(HttpResponseObjectEmpty {
            error: true,
            message: "Error getting total items: Role not valid".to_string(),
        }),
    };
    
    let total_items = match total_items_query.count().get_result::<i64>(conn) {
        Ok(count) => count,
        Err(e) => {
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmpty {
                error: true,
                message: format!("Error getting total items: {}", e),
            });
        }   
    };

    match admin_model::AdminModel::db_read_all_matching_community_by_range(admin_role, conn, per_page, offset) {
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
    security(
        ("Token" = [])
    )
)]
pub async fn get_admin_by_id(id: web::Path<String>, req: HttpRequest, conf: web::Data<Arc<MyCondominiumConfig>>) -> HttpResponse {
    let id = id.into_inner();

    let conn = &mut establish_connection_pg(&conf);

    let id = match Uuid::parse_str(&id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return HttpResponse::BadRequest().json(HttpResponseObjectEmpty {
                error: true,
                message: "Invalid Admin ID format".to_string(),
            });
        }
    };

    let admin_role = match authenticate_user(req.clone(), conn, conf) {
        Ok((role, claims, token)) => {
            if role.role == UserRoles::Root || role.role == UserRoles::Admin {
                role
            } else {
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

    let admin_user = match user_model::UserModel::db_read_by_id(conn, admin_role.user_id) {
        Ok(admin_user) => admin_user,
        Err(e) => {
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmpty {
                error: true,
                message: "Error getting admin user".to_string(),
            });
        }
    };

    let admin = match admin_model::AdminModel::db_read_by_id(conn, admin_user.entity_id) {
        Ok(admin) => admin,
        Err(e) => {
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmpty {
                error: true,
                message: "Error getting admin".to_string(),
            });
        }
    };

    match admin.db_read_by_id_matching_community(id, conn) {
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
