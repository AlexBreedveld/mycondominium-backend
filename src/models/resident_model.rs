use crate::models::lib::DatabaseTrait;
use crate::models::lib::DatabaseTraitVec;
use diesel::{QueryResult, RunQueryDsl};
use crate::schema::residents::dsl::residents;
use super::prelude::*;

#[derive(Queryable, Selectable, Insertable, Serialize, Deserialize, Clone, Debug, AsChangeset, Validate, ToSchema, DbOps)]
#[diesel(table_name = crate::schema::residents)]
pub struct ResidentModel {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    #[validate(length(max = 20, message = "Unit number is too long"))]
    pub unit_number: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub date_of_birth: Option<NaiveDate>,
    pub resident_since: NaiveDateTime,
    pub is_active: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Clone, Debug, Validate, ToSchema)]
pub struct ResidentModelNew {
    pub first_name: String,
    pub last_name: String,
    #[validate(length(max = 20, message = "Unit number is too long"))]
    pub unit_number: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub date_of_birth: Option<NaiveDate>,
    pub resident_since: NaiveDateTime,
    pub is_active: bool,
}