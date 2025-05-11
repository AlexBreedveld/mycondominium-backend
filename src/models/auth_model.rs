use super::prelude::*;
use super::*;

#[derive(Serialize, Deserialize, Clone, Debug, Validate, ToSchema)]
pub struct AuthModel {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub token_id: Uuid,
    pub user_id: Uuid,
    pub exp: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug, Validate, ToSchema)]
pub struct AuthAdminNewSelfServiceModel {
    pub community: community_model::CommunityModelNew,
    pub admin: admin_model::AdminModelNewSelfService,
}

#[derive(Serialize, Deserialize, Clone, Debug, Validate, ToSchema)]
pub struct AuthUserModelResult {
    pub admin: Option<admin_model::AdminModel>,
    pub resident: Option<resident_model::ResidentModel>,
    pub user: UserModelResult,
    pub role: UserRoleModel,
}
