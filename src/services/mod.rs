pub mod admin_service;
pub mod api;
pub mod auth_service;
mod prelude;
pub mod resident_service;

pub use crate::services::prelude::*;

#[derive(OpenApi)]
#[openapi(
    components(
        schemas(
            HttpResponseObjectEmpty,
            HttpResponseObjectEmptyError,
            HttpResponseObjectEmptyEntity
        ),
    ),
    nest(
        (path = "/api/resident", api = resident_service::ResidentApi),
        (path = "/api/admin", api = admin_service::AdminApi),
        (path = "/api/auth", api = auth_service::AuthApi),
    )
)]
pub struct ApiDoc;
