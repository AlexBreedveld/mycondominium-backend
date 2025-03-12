pub mod resident_service;
mod prelude;
pub mod api;

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
    )
)]

pub struct ApiDoc;