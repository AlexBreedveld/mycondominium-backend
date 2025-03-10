use super::prelude::*;

#[derive(Debug, Queryable, Identifiable)]
#[diesel(table_name = invoices)]
pub struct Invoice {
    pub id: Uuid,
    pub resident_id: Uuid,
    pub issue_date: NaiveDate,
    pub due_date: NaiveDate,
    pub amount: BigDecimal,
    pub status: String,                      // max_length 20
    pub paid_date: Option<NaiveDate>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}