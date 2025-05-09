pub mod get_resident;
pub mod invite_resident;
pub mod upsert_resident;

use super::prelude::*;
type ResidentListHttpResponse = HttpResponseObject<Vec<resident_model::ResidentModel>>;
type ResidentGetHttpResponse = HttpResponseObject<resident_model::ResidentModel>;

#[derive(OpenApi)]
#[openapi(
    paths(
        get_resident::get_residents,
        get_resident::count_resident,
        get_resident::get_resident_by_id,
        upsert_resident::new_resident,
        upsert_resident::update_resident,
        upsert_resident::delete_resident,
        invite_resident::new_resident_invite
    ),
    components(schemas(
        resident_model::ResidentModel,
        resident_model::ResidentModelNew,
        resident_model::ResidentModelEdit
    ))
)]
pub struct ResidentApi;
