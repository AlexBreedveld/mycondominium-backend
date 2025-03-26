pub mod api;
mod prelude;
pub mod resident_service;
pub mod admin_service;

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
    )
)]
pub struct ApiDoc;
