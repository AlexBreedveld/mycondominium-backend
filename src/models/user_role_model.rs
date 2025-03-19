use super::prelude::*;

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
)]
#[diesel(table_name = user_roles)]
pub struct UserRoleModel {
    pub id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    pub community_id: Option<Uuid>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
