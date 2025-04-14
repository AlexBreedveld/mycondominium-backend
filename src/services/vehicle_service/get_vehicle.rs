use std::sync::Arc;
use super::*;

#[utoipa::path(
    get,
    tag = "Vehicle",
    path = "/list",
    params(
        ("page" = Option<i64>, Query, description = "Page number for pagination (default: 1)"),
        ("per_page" = Option<i64>, Query, description = "Number of items per page for pagination (default: 10)"),
    ),
    responses(
        (status = 200, description = "Got vehicles successfully", body = VehicleListHttpResponse, headers(
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
pub async fn get_vehicles(query: web::Query<PaginationParams>, req: HttpRequest, conf: web::Data<Arc<MyCondominiumConfig>>) -> HttpResponse {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(10);
    let offset = (page - 1) * per_page;

    let conn = &mut establish_connection_pg(&conf);

    let (role, claims, token) = match authenticate_user(req.clone(), conn) {
        Ok((role, claims, token)) => (role, claims, token),
        Err(_) => {
            return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Unauthorized".to_string(),
            });
        }
    };

    let mut total_items_query = vehicles::table
        .inner_join(residents::table.on(vehicles::resident_id.eq(residents::id)))
        .inner_join(users::table.on(users::entity_id.eq(residents::id)))
        .inner_join(user_roles::table.on(user_roles::user_id.eq(users::id)))
        .into_boxed();

    match role.role {
        UserRoles::Root => (),
        UserRoles::Admin => {
            total_items_query = total_items_query.filter(user_roles::community_id.eq(role.community_id));;
        },
        UserRoles::Resident | UserRoles::Guest => {
            total_items_query = total_items_query.filter(user_roles::user_id.eq(role.user_id));;
        },
    };

    let total_items = match total_items_query.select(vehicle_model::VehicleModel::as_select()).count().get_result::<i64>(conn) {
        Ok(count) => count,
        Err(e) => {
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmpty {
                error: true,
                message: format!("Error getting total items: {}", e),
            });
        }
    };

    match vehicle_model::VehicleModel::db_read_all_matching_community_by_range(role, conn, per_page, offset) {
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
                    message: "Got vehicles successfully".to_string(),
                    object: Some(res),
                })
        }
        Err(e) => HttpResponse::InternalServerError().json(HttpResponseObjectEmpty {
            error: true,
            message: format!("Error getting vehicles: {}", e),
        }),
    }
}

#[utoipa::path(
    get,
    tag = "Vehicle",
    path = "/get/{id}",
    params(
        ("id" = Uuid, Path, description = "Vehicle ID"),
    ),
    responses(
        (status = 200, description = "Got Vehicle successfully", body = VehicleGetHttpResponse),
        (status = 400, description = "Invalid Vehicle ID format or Vehicle ID is required", body = HttpResponseObjectEmptyError),
        (status = 401, description = "Unauthorized", body = HttpResponseObjectEmptyError),
        (status = 500, description = "Internal server error", body = HttpResponseObjectEmptyError)
    ),
    security(
        ("Token" = [])
    )
)]
pub async fn get_vehicle_by_id(id: web::Path<String>, req: HttpRequest, conf: web::Data<Arc<MyCondominiumConfig>>) -> HttpResponse {
    let id = id.into_inner();

    let conn = &mut establish_connection_pg(&conf);

    let id = match Uuid::parse_str(&id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return HttpResponse::BadRequest().json(HttpResponseObjectEmpty {
                error: true,
                message: "Invalid Vehicle ID format".to_string(),
            });
        }
    };

    let (role, claims, token) = match authenticate_user(req.clone(), conn) {
        Ok((role, claims, token)) => (role, claims, token),
        Err(_) => {
            return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Unauthorized".to_string(),
            });
        }
    };
    
    match vehicle_model::VehicleModel::db_read_by_id_matching_community(role, conn, id) {
        Ok(res) => HttpResponse::Ok().json(HttpResponseObject {
            error: false,
            message: "Got Vehicle successfully".to_string(),
            object: Some(res),
        }),
        Err(e) => HttpResponse::InternalServerError().json(HttpResponseObjectEmpty {
            error: true,
            message: "Error getting vehicles".to_string(),
        })
    }
}