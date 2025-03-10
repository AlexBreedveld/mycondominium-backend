use super::prelude::*;

#[derive(Debug, Queryable, Identifiable)]
#[diesel(table_name = reservations)]
pub struct Reservation {
    pub id: Uuid,
    pub resident_id: Uuid,
    pub common_area_id: Uuid,
    pub reservation_date: NaiveDateTime,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    pub status: String,                      // max_length 20
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}