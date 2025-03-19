use super::prelude::*;

#[derive(Debug, Queryable, Identifiable, Validate)]
#[diesel(table_name = announcements)]
pub struct AnnouncementModel {
    pub id: Uuid,
    #[validate(length(max = 150, message = "Title is too long"))]
    pub title: String,
    pub community_id: Option<Uuid>,
    pub message: String,
    pub sent_at: NaiveDateTime,
}
