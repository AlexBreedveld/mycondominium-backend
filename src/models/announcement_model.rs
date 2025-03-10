use super::prelude::*;

#[derive(Debug, Queryable, Identifiable)]
#[diesel(table_name = announcements)]
pub struct Announcement {
    pub id: Uuid,
    pub title: String,       // max_length 150
    pub message: String,
    pub sent_at: NaiveDateTime,
}
