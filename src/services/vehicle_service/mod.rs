pub mod get_vehicle;
pub mod upsert_vehicle;

use super::prelude::*;
type VehicleListHttpResponse = HttpResponseObject<Vec<vehicle_model::VehicleModel>>;
type VehicleGetHttpResponse = HttpResponseObject<vehicle_model::VehicleModel>;

#[derive(OpenApi)]
#[openapi(
    paths(
        get_vehicle::get_vehicles,
        get_vehicle::get_vehicle_by_id,
        upsert_vehicle::new_vehicle,
        upsert_vehicle::update_vehicle,
        upsert_vehicle::delete_vehicle,
    ),
    components(schemas(vehicle_model::VehicleModel, vehicle_model::VehicleModelNew))
)]
pub struct VehicleApi;