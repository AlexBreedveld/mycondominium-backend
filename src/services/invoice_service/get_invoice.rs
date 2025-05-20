use super::*;

#[utoipa::path(
    get,
    tag = "Invoice",
    path = "/list",
    params(
        ("page" = Option<i64>, Query, description = "Page number for pagination (default: 1)"),
        ("per_page" = Option<i64>, Query, description = "Number of items per page for pagination (default: 10)"),
    ),
    responses(
        (status = 200, description = "Got Invoices successfully", body = InvoiceListHttpResponse, headers(
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
pub async fn get_invoices(
    query: web::Query<PaginationParams>,
    req: HttpRequest,
    conf: web::Data<Arc<MyCondominiumConfig>>,
) -> HttpResponse {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(10);
    let offset = (page - 1) * per_page;

    let conn = &mut establish_connection_pg(&conf);

    let (role, _claims, _token) = match authenticate_user(req.clone(), conn, conf) {
        Ok((role, claims, token)) => (role, claims, token),
        Err(_) => {
            return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Unauthorized".to_string(),
            });
        }
    };

    let total_items = match invoice_model::InvoiceModel::db_count_all_matching(role.clone(), conn) {
        Ok(res) => res,
        Err(e) => {
            log::error!("Error getting Invoices: {}", e);
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Error getting Invoices".to_string(),
            });
        }
    };

    match invoice_model::InvoiceModel::db_read_all_matching_by_range(role, conn, per_page, offset) {
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
                    message: "Got Invoices successfully".to_string(),
                    object: Some(res),
                })
        }
        Err(e) => {
            log::error!("Error getting Invoices: {}", e);
            HttpResponse::InternalServerError().json(HttpResponseObjectEmpty {
                error: true,
                message: format!("Error getting Invoices: {}", e),
            })
        }
    }
}

#[utoipa::path(
    get,
    tag = "Invoice",
    path = "/get/{id}",
    params(
        ("id" = Uuid, Path, description = "Invoice ID"),
    ),
    responses(
        (status = 200, description = "Got Invoice successfully", body = InvoiceGetHttpResponse),
        (status = 400, description = "Invalid Invoice ID format or Invoice ID is required", body = HttpResponseObjectEmptyError),
        (status = 401, description = "Unauthorized", body = HttpResponseObjectEmptyError),
        (status = 500, description = "Internal server error", body = HttpResponseObjectEmptyError)
    ),
    security(
        ("Token" = [])
    )
)]
pub async fn get_invoice_by_id(
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
                message: "Invalid Invoice ID format".to_string(),
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

    match invoice_model::InvoiceModel::db_read_by_id_matching_resident(role, conn, id) {
        Ok(user_req) => HttpResponse::Ok().json(HttpResponseObject {
            error: false,
            message: "Got Invoice successfully".to_string(),
            object: Some(user_req),
        }),
        Err(e) => {
            log::error!("Error getting Invoice: {}", e);
            HttpResponse::InternalServerError().json(HttpResponseObjectEmpty {
                error: true,
                message: format!("Error getting Invoice: {}", e),
            })
        }
    }
}
