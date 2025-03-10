use super::prelude::*;

#[derive(Debug, Queryable, Identifiable)]
#[diesel(table_name = vehicles)]
pub struct Vehicle {
    pub id: Uuid,
    pub resident_id: Uuid,
    pub license_plate: String,               // max_length 20
    pub model: Option<String>,               // max_length 100
    pub color: Option<String>,               // max_length 50
    pub created_at: NaiveDateTime,
}