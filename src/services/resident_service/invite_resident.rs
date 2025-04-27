use super::*;

#[utoipa::path(
    post,
    tag = "Resident",
    path = "/invite/new",
    request_body = resident_model::ResidentInviteModelNew,
    responses (
        (status = 200, description = "New Resident invited successfully", body = HttpResponseObjectEmpty),
        (status = 401, description = "Unauthorized", body = HttpResponseObjectEmptyError),
        (status = 500, description = "Error deleting Resident", body = HttpResponseObjectEmptyError),
    ),
    security(
        ("Token" = [])
    )
)]
pub async fn new_resident_invite(
    body: web::Json<resident_model::ResidentInviteModelNew>,
    req: HttpRequest,
    conf: web::Data<Arc<MyCondominiumConfig>>,
) -> HttpResponse {
    todo!()
}
