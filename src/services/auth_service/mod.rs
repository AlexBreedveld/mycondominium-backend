pub mod auth;
pub mod new_admin_self_service;
pub mod password_reset;
pub mod sign_in;

use super::prelude::*;

#[derive(OpenApi)]
#[openapi(
    paths(
        sign_in::sign_in,
        auth::auth,
        auth::sign_out,
        new_admin_self_service::new_admin_self_service,
        password_reset::password_reset,
        password_reset::request_password_reset,
    ),
    components(schemas(
        auth_model::AuthModel,
        auth_model::AuthAdminNewSelfServiceModel,
        auth_model::AuthUserModelResult,
        auth_model::PasswordResetModel,
        auth_model::PasswordResetChangeModel,
        auth_model::PasswordResetRequestModel
    ))
)]
pub struct AuthApi;
