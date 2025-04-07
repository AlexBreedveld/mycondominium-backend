use super::prelude::*;
use crate::internal::roles::UserRoles;

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

impl UserRoleModel {
    pub fn count_root_admins(conn: &mut PgConnection) -> QueryResult<i64> {
        crate::schema::user_roles::table
            .filter(crate::schema::user_roles::role.eq("Root".to_string()))
            .count()
            .get_result::<i64>(conn)
    }
}
