use validator_derive::Validate;
use crate::internal::roles::UserRoles;
use crate::services::{Deserialize, Serialize, ToSchema};

#[derive(Serialize, Deserialize, Clone, Debug, Validate, ToSchema)]
pub struct AuthModel {
    pub email: String,
    pub password: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub token_id: uuid::Uuid,
    pub user_id: String,
    pub exp: usize,
}