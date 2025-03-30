pub mod sign_in;

use super::prelude::*;

#[derive(OpenApi)]
#[openapi(
    paths(
        sign_in::sign_in,
    ),
    components(schemas(auth_model::AuthModel))
)]
pub struct AuthApi;