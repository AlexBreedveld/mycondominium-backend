use crate::internal::roles::UserRoles;
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
    DbOps,
)]
#[diesel(table_name = crate::schema::user_roles)]
pub struct UserRoleModel {
    pub id: Uuid,
    pub user_id: Uuid,
    pub role: UserRoles,
    pub community_id: Option<Uuid>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
