use super::prelude::*;

#[derive(Debug, Queryable, Identifiable, Validate)]
#[diesel(table_name = residents)]
pub struct Resident {
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