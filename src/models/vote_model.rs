use super::prelude::*;

#[derive(Debug, Queryable, Identifiable)]
#[diesel(table_name = votes)]
pub struct Vote {
    pub id: Uuid,
    pub election_id: Uuid,
    pub resident_id: Uuid,
    pub vote_option: String,                 // max_length 50
    pub voted_at: NaiveDateTime,
}