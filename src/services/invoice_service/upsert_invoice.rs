use super::*;

#[utoipa::path(
    post,
    tag = "Invoice",
    path = "/new",
    request_body = invoice_model::InvoiceModelNew,
    responses (
        (status = 200, description = "Invoice added successfully", body = HttpResponseObjectEmptyEntity),
        (status = 400, description = "Error adding Invoice", body = HttpResponseObjectEmptyError),
        (status = 401, description = "Unauthorized", body = HttpResponseObjectEmptyError),
        (status = 500, description = "Error adding Invoice", body = HttpResponseObjectEmptyError),
    ),
    security(
        ("Token" = [])
    )
)]
pub async fn new_invoice(
    body: web::Json<invoice_model::InvoiceModelNew>,
    req: HttpRequest,
    conf: web::Data<Arc<MyCondominiumConfig>>,
) -> HttpResponse {
    let conn = &mut establish_connection_pg(&conf);

    let body = body.into_inner();

    let (role, _claims, _token) = match authenticate_user(req.clone(), conn, conf.clone()) {
        Ok((role, claims, token)) => {
            if role.role == UserRoles::Root || role.role == UserRoles::Admin {
                (role, claims, token)
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

    let resident = match resident_model::ResidentModel::db_get_user(conn, body.resident_id) {
        Ok(resident) => resident,
        Err(_) => {
            return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Unauthorized".to_string(),
            });
        }
    };

    if !(role.role == UserRoles::Root
        || (role.role == UserRoles::Admin && role.community_id == resident.role.community_id))
    {
        return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
            error: true,
            message: "Unauthorized".to_string(),
        });
    }

    if let Err(validation_errors) = body.validate() {
        return HttpResponse::BadRequest().json(validation_errors);
    }

    let new_obj = invoice_model::InvoiceModel {
        id: invoice_model::InvoiceModel::new_id(conn),
        resident_id: body.resident_id,
        community_id: body.community_id,
        issue_date: body.issue_date,
        due_date: body.due_date,
        amount: body.amount,
        status: body.status,
        paid_date: body.paid_date,
        created_at: chrono::Utc::now().naive_utc(),
        updated_at: chrono::Utc::now().naive_utc(),
    };

    match new_obj.db_insert(conn) {
        Ok(_) => (),
        Err(e) => {
            log::error!("Error creating Invoice: {}", e);
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmptyError {
                error: true,
                message: format!("Error creating Invoice: {}", e),
            });
        }
    };

    HttpResponse::Ok().json(HttpResponseObjectEmptyEntity {
        error: false,
        message: "Invoice created successfully".to_string(),
        entity_id: Some(new_obj.id),
    })
}

#[utoipa::path(
    put,
    tag = "Invoice",
    path = "/update/{id}",
    request_body = invoice_model::InvoiceModelNew,
    params(
        ("id" = Uuid, Path, description = "Invoice ID"),
    ),
    responses (
        (status = 200, description = "Invoice updated successfully", body = HttpResponseObjectEmpty),
        (status = 400, description = "Error updating Invoice", body = HttpResponseObjectEmptyError),
        (status = 401, description = "Unauthorized", body = HttpResponseObjectEmptyError),
        (status = 500, description = "Internal server error", body = HttpResponseObjectEmptyError),
    ),
    security(
        ("Token" = [])
    )
)]
pub async fn update_invoice(
    id: web::Path<String>,
    body: web::Json<invoice_model::InvoiceModelNew>,
    req: HttpRequest,
    conf: web::Data<Arc<MyCondominiumConfig>>,
) -> HttpResponse {
    let conn = &mut establish_connection_pg(&conf);
    let body = body.into_inner();

    let (role, _claims, _token) = match authenticate_user(req.clone(), conn, conf.clone()) {
        Ok((role, claims, token)) => {
            if role.role == UserRoles::Root || role.role == UserRoles::Admin {
                (role, claims, token)
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

    let id = match Uuid::parse_str(&id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return HttpResponse::BadRequest().json(HttpResponseObjectEmpty {
                error: true,
                message: "Invalid Invoice ID format".to_string(),
            });
        }
    };

    let curr_obj = match invoice_model::InvoiceModel::db_read_by_id(conn, id) {
        Ok(res) => res,
        Err(e) => {
            log::error!("Error updating Invoice: {}", e);
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmpty {
                error: true,
                message: "Error updating Invoice".to_string(),
            });
        }
    };

    let resident = match resident_model::ResidentModel::db_get_user(conn, body.resident_id) {
        Ok(resident) => resident,
        Err(_) => {
            return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Unauthorized".to_string(),
            });
        }
    };

    match role.role {
        UserRoles::Root => {}
        UserRoles::Admin => {
            if role.community_id != resident.role.community_id
                || body.community_id != role.community_id.unwrap()
            {
                return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
                    error: true,
                    message: "Unauthorized".to_string(),
                });
            }
        }
        _ => {
            return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Unauthorized".to_string(),
            });
        }
    }

    if let Err(validation_errors) = body.validate() {
        return HttpResponse::BadRequest().json(validation_errors);
    }

    let new_obj = invoice_model::InvoiceModel {
        id: curr_obj.id,
        resident_id: body.resident_id,
        community_id: body.community_id,
        issue_date: body.issue_date,
        due_date: body.due_date,
        amount: body.amount,
        status: body.status,
        paid_date: body.paid_date,
        created_at: curr_obj.created_at,
        updated_at: chrono::Utc::now().naive_utc(),
    };

    match new_obj.db_update(conn) {
        Ok(_) => HttpResponse::Ok().json(HttpResponseObjectEmpty {
            error: false,
            message: "Invoice updated successfully".to_string(),
        }),
        Err(e) => {
            log::error!("Error updating Invoice: {}", e);
            HttpResponse::Ok().json(HttpResponseObjectEmpty {
                error: true,
                message: format!("Error updating Invoice: {}", e),
            })
        }
    }
}

#[utoipa::path(
    delete,
    tag = "Invoice",
    path = "/delete/{id}",
    params(
        ("id" = Uuid, Path, description = "Invoice ID"),
    ),
    responses (
        (status = 200, description = "Invoice deleted successfully", body = HttpResponseObjectEmpty),
        (status = 400, description = "Invalid Invoice ID format", body = HttpResponseObjectEmptyError),
        (status = 401, description = "Unauthorized", body = HttpResponseObjectEmptyError),
        (status = 500, description = "Error deleting Invoice", body = HttpResponseObjectEmptyError),
    ),
    security(
        ("Token" = [])
    )
)]
pub async fn delete_invoice(
    id: web::Path<String>,
    req: HttpRequest,
    conf: web::Data<Arc<MyCondominiumConfig>>,
) -> HttpResponse {
    let conn = &mut establish_connection_pg(&conf);

    let (role, _claims, _token) = match authenticate_user(req.clone(), conn, conf.clone()) {
        Ok((role, claims, token)) => {
            if role.role == UserRoles::Root || role.role == UserRoles::Admin {
                (role, claims, token)
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

    let id = match Uuid::parse_str(&id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return HttpResponse::BadRequest().json(HttpResponseObjectEmpty {
                error: true,
                message: "Invalid Invoice ID format".to_string(),
            });
        }
    };

    let curr_obj = match invoice_model::InvoiceModel::db_read_by_id(conn, id) {
        Ok(user_req) => user_req,
        Err(e) => {
            log::error!("Error deleting Invoice: {}", e);
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmpty {
                error: true,
                message: "Error deleting Invoice".to_string(),
            });
        }
    };

    let resident = match resident_model::ResidentModel::db_get_user(conn, curr_obj.resident_id) {
        Ok(resident) => resident,
        Err(_) => {
            return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Unauthorized".to_string(),
            });
        }
    };

    if !(role.role == UserRoles::Root
        || (role.role == UserRoles::Admin && role.community_id == resident.role.community_id))
    {
        return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
            error: true,
            message: "Unauthorized".to_string(),
        });
    }

    match invoice_model::InvoiceModel::db_delete_by_id(conn, id) {
        Ok(_) => HttpResponse::Ok().json(HttpResponseObjectEmpty {
            error: false,
            message: "Invoice deleted successfully".to_string(),
        }),
        Err(e) => {
            log::error!("Error deleting Invoice: {}", e);
            HttpResponse::Ok().json(HttpResponseObjectEmpty {
                error: true,
                message: format!("Error deleting Invoice: {}", e),
            })
        }
    }
}
