pub mod auth;
pub mod new_admin_self_service;
pub mod sign_in;

use super::prelude::*;

#[derive(OpenApi)]
#[openapi(
    paths(
        sign_in::sign_in,
        auth::auth,
        auth::sign_out,
        new_admin_self_service::new_admin_self_service
    ),
    components(schemas(
        auth_model::AuthModel,
        auth_model::AuthAdminNewSelfServiceModel,
        auth_model::AuthUserModelResult
    ))
)]
pub struct AuthApi;
