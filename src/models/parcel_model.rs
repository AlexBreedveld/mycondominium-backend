use super::prelude::*;

#[derive(Debug, Queryable, Identifiable)]
#[diesel(table_name = parcels)]
pub struct Parcel {
    pub id: Uuid,
    pub resident_id: Uuid,
    pub parcel_type: String,                 // max_length 50
    pub description: Option<String>,
    pub arrival_date: NaiveDateTime,
    pub received: bool,
    pub received_at: Option<NaiveDateTime>,
}