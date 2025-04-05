use super::prelude::*;
use crate::internal::user_types::UserTypes;

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
#[diesel(table_name = crate::schema::users)]
pub struct UserModel {
    pub id: Uuid,
    pub entity_id: Uuid,
    pub entity_type: UserTypes,
    pub admin_id: Option<Uuid>,
    pub resident_id: Option<Uuid>,
    pub password: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Clone, Debug, Validate, ToSchema)]
pub struct UserModelResult {
    pub id: Uuid,
    pub entity_id: Uuid,
    pub entity_type: UserTypes,
    pub admin_id: Option<Uuid>,
    pub resident_id: Option<Uuid>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}