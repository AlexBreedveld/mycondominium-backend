use crate::internal::roles::UserRoles;
use crate::models::admin_model::AdminModelNew;
use crate::models::community_model::CommunityModelNew;
use crate::models::user_role_model::UserRoleModel;
use crate::services::{Deserialize, Serialize, ToSchema};
use validator_derive::Validate;

#[derive(Serialize, Deserialize, Clone, Debug, Validate, ToSchema)]
pub struct AuthModel {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub token_id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub exp: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug, Validate, ToSchema)]
pub struct AuthAdminNewSelfServiceModel {
    pub community: CommunityModelNew,
    pub admin: AdminModelNew,
}
