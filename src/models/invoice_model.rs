use super::prelude::*;

#[derive(Debug, Queryable, Identifiable, Validate)]
#[diesel(table_name = invoices)]
pub struct InvoiceModel {
    pub id: Uuid,
    pub resident_id: Uuid,
    pub community_id: Option<Uuid>,
    pub issue_date: NaiveDate,
    pub due_date: NaiveDate,
    pub amount: BigDecimal,
    #[validate(length(max = 20, message = "Status is too long"))]
    pub status: String, // max_length 20
    pub paid_date: Option<NaiveDate>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
