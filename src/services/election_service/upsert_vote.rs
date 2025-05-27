use super::*;

#[utoipa::path(
    post,
    tag = "Election",
    path = "/vote",
    request_body = election_model::ElectionVotesModelNew,
    responses (
        (status = 200, description = "Election vote added successfully", body = HttpResponseObjectEmptyEntity),
        (status = 400, description = "Error adding Election vote", body = HttpResponseObjectEmptyError),
        (status = 401, description = "Unauthorized", body = HttpResponseObjectEmptyError),
        (status = 500, description = "Error adding Election vote", body = HttpResponseObjectEmptyError),
    ),
    security(
        ("Token" = [])
    )
)]
pub async fn new_election_vote(
    body: web::Json<election_model::ElectionVotesModelNew>,
    req: HttpRequest,
    conf: web::Data<Arc<MyCondominiumConfig>>,
) -> HttpResponse {
    let conn = &mut establish_connection_pg(&conf);

    let body = body.into_inner();

    let (role, _claims, _token) = match authenticate_user(req.clone(), conn, conf.clone()) {
        Ok((role, claims, token)) => (role, claims, token),
        Err(_) => {
            return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Unauthorized".to_string(),
            });
        }
    };

    let election = match election_model::ElectionModel::db_read_by_id(conn, body.election_id) {
        Ok(elec) => elec,
        Err(_) => {
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmptyError {
                error: true,
                message: "Error while getting election".to_string(),
            });
        }
    };

    if !(role.role == UserRoles::Root
        || (role.role == UserRoles::Admin && role.community_id == Some(election.community_id)))
        || (role.role == UserRoles::Resident && role.community_id == Some(election.community_id))
    {
        return HttpResponse::Unauthorized().json(HttpResponseObjectEmptyError {
            error: true,
            message: "Unauthorized".to_string(),
        });
    }

    if let Err(validation_errors) = body.validate() {
        return HttpResponse::BadRequest().json(validation_errors);
    }

    let new_obj = election_model::ElectionVotesModel {
        id: election_model::ElectionVotesModel::new_id(conn),
        election_id: body.election_id,
        user_id: role.user_id,
        vote_option: body.vote_option,
        voted_at: chrono::Utc::now().naive_utc(),
    };

    match new_obj.db_insert(conn) {
        Ok(_) => (),
        Err(e) => {
            log::error!("Error creating Election vote: {}", e);
            return HttpResponse::InternalServerError().json(HttpResponseObjectEmptyError {
                error: true,
                message: format!("Error creating Election vote: {}", e),
            });
        }
    };

    HttpResponse::Ok().json(HttpResponseObjectEmptyEntity {
        error: false,
        message: "Election vote created successfully".to_string(),
        entity_id: Some(new_obj.id),
    })
}
