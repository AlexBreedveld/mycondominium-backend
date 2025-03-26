use diesel::{QueryResult, RunQueryDsl};
use crate::schema::auth_tokens::dsl::auth_tokens;
use super::prelude::*;

#[derive(Queryable, Selectable, Insertable, Serialize, Deserialize, Clone, Debug, AsChangeset, Validate, ToSchema, DbOps)]
#[diesel(table_name = crate::schema::auth_tokens)]
pub struct AuthTokenModel {
    pub user_id: Uuid,
    pub id: Uuid,
    pub time_added: NaiveDateTime,
    pub active: bool,
    pub time_last_used: NaiveDateTime,
    pub device: Option<String>,
    pub browser: Option<String>,
    pub version: Option<String>,
    pub cpu_arch: Option<String>,
}