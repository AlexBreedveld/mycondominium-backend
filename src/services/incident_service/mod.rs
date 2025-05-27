pub mod get_incident;
pub mod upsert_incident;

use super::prelude::*;
type IncidentListHttpResponse = HttpResponseObject<Vec<incident_model::IncidentModel>>;
type IncidentGetHttpResponse = HttpResponseObject<incident_model::IncidentModel>;

#[derive(OpenApi)]
#[openapi(
    paths(
        get_incident::get_incidents,
        get_incident::get_incident_by_id,
        upsert_incident::new_incident,
        upsert_incident::update_incident,
        upsert_incident::delete_incident,
    ),
    components(schemas(
        incident_model::IncidentModel,
        incident_model::IncidentModelNew,
        incident_model::IncidentStatus
    ))
)]
pub struct IncidentApi;
