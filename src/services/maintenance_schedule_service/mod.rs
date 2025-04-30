pub mod get_maintenance_schedule;
pub mod upsert_maintenance_schedule;

use super::prelude::*;
type MaintenanceScheduleListHttpResponse =
    HttpResponseObject<Vec<maintenance_schedule_model::MaintenanceScheduleModel>>;
type MaintenanceScheduleGetHttpResponse =
    HttpResponseObject<maintenance_schedule_model::MaintenanceScheduleModel>;

#[derive(OpenApi)]
#[openapi(
    paths(
        get_maintenance_schedule::get_maintenance_schedules,
        get_maintenance_schedule::count_maintenance_schedule,
        get_maintenance_schedule::get_maintenance_schedule_by_id,
        upsert_maintenance_schedule::new_maintenance_schedule,
        upsert_maintenance_schedule::update_maintenance_schedule,
        upsert_maintenance_schedule::delete_maintenance_schedule,
    ),
    components(schemas(
        maintenance_schedule_model::MaintenanceScheduleModel,
        maintenance_schedule_model::MaintenanceScheduleModelNew,
        maintenance_schedule_model::MaintenanceScheduleStatus,
    ))
)]
pub struct MaintenanceScheduleApi;
