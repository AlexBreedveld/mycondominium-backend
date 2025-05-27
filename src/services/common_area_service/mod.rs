pub mod get_common_area;
pub mod upsert_common_area;

use super::prelude::*;

type CommonAreaListHttpResponse = HttpResponseObject<Vec<common_area_model::CommonAreaModel>>;
type CommonAreaGetHttpResponse = HttpResponseObject<common_area_model::CommonAreaModel>;

#[derive(OpenApi)]
#[openapi(
    paths(
        get_common_area::get_common_areas,
        get_common_area::get_common_area_by_id,
        upsert_common_area::new_common_area,
        upsert_common_area::update_common_area,
        upsert_common_area::delete_common_area,
    ),
    components(schemas(
        common_area_model::CommonAreaModel,
        common_area_model::CommonAreaModelNew
    ))
)]
pub struct CommonAreaApi;
