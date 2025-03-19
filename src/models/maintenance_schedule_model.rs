use super::prelude::*;

#[derive(Debug, Queryable, Identifiable, Validate)]
#[diesel(table_name = maintenance_schedules)]
pub struct MaintenanceScheduleModel {
    pub id: Uuid,
    pub community_id: Option<Uuid>,
    pub description: String,
    pub scheduled_date: NaiveDateTime,
    #[validate(length(max = 20, message = "Status is too long"))]
    pub status: String,
    pub details: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
