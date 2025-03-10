use super::prelude::*;

#[derive(Debug, Queryable, Identifiable)]
#[diesel(table_name = residents)]
pub struct Resident {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub unit_number: Option<String>,         // max_length 20
    pub address: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub date_of_birth: Option<NaiveDate>,
    pub resident_since: NaiveDateTime,
    pub is_active: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}