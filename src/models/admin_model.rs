use diesel::{QueryResult, RunQueryDsl};
use crate::schema::admins::dsl::admins;
use super::prelude::*;

#[derive(Queryable, Selectable, Insertable, Serialize, Deserialize, Clone, Debug, AsChangeset, Validate, ToSchema)]
#[diesel(table_name = crate::schema::admins)]
pub struct AdminModel {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Queryable, Selectable, Insertable, Serialize, Deserialize, Clone, Debug, AsChangeset, Validate, ToSchema)]
#[diesel(table_name = crate::schema::admins)]
pub struct AdminModelNew {
    pub first_name: String,
    pub last_name: String,
    pub phone: Option<String>,
    pub email: Option<String>,
}