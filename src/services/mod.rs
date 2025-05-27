pub mod admin_service;
pub mod api;
pub mod auth_service;
pub mod common_area_service;
pub mod community_service;
pub mod election_service;
pub mod incident_service;
pub mod invoice_service;
pub mod maintenance_schedule_service;
pub mod parcel_service;
pub mod prelude;
pub mod reservation_service;
pub mod resident_service;
pub mod vehicle_service;

pub use prelude::*;
use utoipa::openapi::security::{ApiKey, ApiKeyValue};

#[derive(OpenApi)]
#[openapi(
    components(
        schemas(
            HttpResponseObjectEmpty,
            HttpResponseObjectEmptyError,
            HttpResponseObjectEmptyEntity
        ),
    ),
    modifiers(&SecurityAddon),
    nest(
        (path = "/api/resident", api = resident_service::ResidentApi),
        (path = "/api/admin", api = admin_service::AdminApi),
        (path = "/api/auth", api = auth_service::AuthApi),
        (path = "/api/common_area", api = common_area_service::CommonAreaApi),
        (path = "/api/community", api = community_service::CommunityApi),
        (path = "/api/vehicle", api = vehicle_service::VehicleApi),
        (path = "/api/maintenance_schedule", api = maintenance_schedule_service::MaintenanceScheduleApi),
        (path = "/api/parcel", api = parcel_service::ParcelApi),
        (path = "/api/reservation", api = reservation_service::ReservationApi),
        (path = "/api/invoice", api = invoice_service::InvoiceApi),
        (path = "/api/incident", api = incident_service::IncidentApi),
        (path = "/api/election", api = election_service::ElectionApi),
    )
)]
pub struct ApiDoc;

struct SecurityAddon;
impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        // NOTE: we can unwrap safely since there already is components registered.
        let components = openapi.components.as_mut().unwrap();
        components.add_security_scheme(
            "Token",
            utoipa::openapi::security::SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new(
                "X-Auth-Token",
            ))),
        );
    }
}
