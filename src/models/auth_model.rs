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

#[derive(Serialize, Deserialize, Clone, Debug, Validate, ToSchema)]
pub struct PasswordResetRequestModel {
    pub email: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Validate, ToSchema)]
pub struct PasswordResetChangeModel {
    pub token: String,
    pub password: String,
}

#[derive(
    Queryable,
    Selectable,
    Insertable,
    Serialize,
    Deserialize,
    Clone,
    Debug,
    AsChangeset,
    Validate,
    ToSchema,
    DbOps,
)]
#[diesel(table_name = crate::schema::password_reset)]
pub struct PasswordResetModel {
    pub id: Uuid,
    pub email: String,
    pub user_id: Uuid,
    pub token: String,
    pub created_at: NaiveDateTime,
}
