use super::prelude::*;

#[derive(Debug, Queryable, Identifiable, Validate)]
#[diesel(table_name = parcels)]
pub struct Parcel {
    pub id: Uuid,
    pub resident_id: Uuid,
    #[validate(length(max = 50, message = "Type is too long"))]
    pub parcel_type: String,
    pub description: Option<String>,
    pub arrival_date: NaiveDateTime,
    pub received: bool,
    pub received_at: Option<NaiveDateTime>,
}