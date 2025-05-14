use super::prelude::*;

#[derive(Debug, Queryable, Identifiable, Validate)]
#[diesel(table_name = reservations)]
pub struct ReservationModel {
    pub id: Uuid,
    pub resident_id: Uuid,
    pub common_area_id: Uuid,
    pub reservation_date: NaiveDateTime,
    pub start_time: NaiveDateTime,
    pub end_time: NaiveDateTime,
    #[validate(length(max = 20, message = "Status is too long"))]
    pub status: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
