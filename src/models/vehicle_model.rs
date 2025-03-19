use super::prelude::*;

#[derive(Debug, Queryable, Identifiable, Validate)]
#[diesel(table_name = vehicles)]
pub struct VehicleModel {
    pub id: Uuid,
    pub resident_id: Uuid,
    #[validate(length(max = 20, message = "License plate is too long"))]
    pub license_plate: String,
    #[validate(length(max = 100, message = "Model is too long"))]
    pub model: Option<String>,
    #[validate(length(max = 50, message = "Color is too long"))]
    pub color: Option<String>,
    pub created_at: NaiveDateTime,
}
