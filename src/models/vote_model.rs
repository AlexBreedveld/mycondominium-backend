use super::prelude::*;

#[derive(Debug, Queryable, Identifiable, Validate)]
#[diesel(table_name = votes)]
pub struct VoteModel {
    pub id: Uuid,
    pub election_id: Uuid,
    pub resident_id: Uuid,
    #[validate(length(max = 50, message = "Option is too long"))]
    pub vote_option: String,
    pub voted_at: NaiveDateTime,
}
