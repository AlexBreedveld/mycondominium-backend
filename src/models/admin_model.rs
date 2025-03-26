use super::prelude::*;
use db_ops_derive::DbOps;
use diesel::{QueryResult, RunQueryDsl};

#[derive(
    Queryable,
    Selectable,
    Insertable,
    Serialize,
    Deserialize,
    DbOps,
    Clone,
    Debug,
    AsChangeset,
    Validate,
    ToSchema,
)]
#[diesel(table_name = crate::schema::admins)]
pub struct AdminModel {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub phone: Option<String>,
    pub email: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Clone, Debug, Validate, ToSchema)]
pub struct AdminModelNew {
    pub first_name: String,
    pub last_name: String,
    pub phone: Option<String>,
    pub email: String,
}
