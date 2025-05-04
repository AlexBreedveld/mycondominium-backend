pub mod get_parcel;
pub mod upsert_parcel;

use super::prelude::*;
type ParcelListHttpResponse = HttpResponseObject<Vec<parcel_model::ParcelModel>>;
type ParcelGetHttpResponse = HttpResponseObject<parcel_model::ParcelModel>;

#[derive(OpenApi)]
#[openapi(
    paths(
        get_parcel::get_parcels,
        get_parcel::get_parcel_by_id,
        upsert_parcel::new_parcel,
        upsert_parcel::update_parcel,
        upsert_parcel::delete_parcel,
    ),
    components(schemas(parcel_model::ParcelModel, parcel_model::ParcelModelNew,))
)]
pub struct ParcelApi;
