use super::prelude::*;

#[derive(
    Queryable,
    Selectable,
    Insertable,
    Serialize,
    Deserialize,
    Clone,
    Debug,
    AsChangeset,
    Validate,
    ToSchema,
)]
#[diesel(table_name = document_shares)]
pub struct DocumentShareModel {
    pub id: Uuid,
    pub user_id: Uuid,
    pub document_id: Uuid,
    pub read_only: bool,
}
