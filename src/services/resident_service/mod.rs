pub mod get_resident;
mod upsert_resident;

use super::prelude::*;
type ResidentListHttpResponse = HttpResponseObject<Vec<resident_model::ResidentModel>>;
type ResidentGetHttpResponse = HttpResponseObject<resident_model::ResidentModel>;

#[derive(OpenApi)]
#[openapi(
    paths(
        get_resident::get_residents,
        get_resident::get_resident_by_id,
    ),
    components(schemas(resident_model::ResidentModel))
)]
pub struct ResidentApi;
