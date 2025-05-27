use super::prelude::*;
use super::*;

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
    DbOps,
)]
#[diesel(table_name = elections)]
pub struct ElectionModel {
    pub id: Uuid,
    pub community_id: Uuid,
    #[validate(length(max = 150, message = "Title is too long"))]
    pub title: String,
    pub description: Option<String>,
    pub start_date: NaiveDateTime,
    pub end_date: NaiveDateTime,
    pub created_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Clone, Debug, Validate, ToSchema)]
pub struct ElectionModelNew {
    pub community_id: Uuid,
    #[validate(length(max = 150, message = "Title is too long"))]
    pub title: String,
    pub description: Option<String>,
    pub start_date: NaiveDateTime,
    pub end_date: NaiveDateTime,
}

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
    DbOps,
)]
#[diesel(table_name = election_candidates)]
pub struct ElectionCandidatesModel {
    pub id: Uuid,
    pub election_id: Uuid,
    #[validate(length(max = 150, message = "Name is too long"))]
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Validate, ToSchema)]
pub struct ElectionCandidatesModelNew {
    pub election_id: Uuid,
    #[validate(length(max = 150, message = "Name is too long"))]
    pub name: String,
}

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
    DbOps,
)]
#[diesel(table_name = votes)]
pub struct ElectionVotesModel {
    pub id: Uuid,
    pub election_id: Uuid,
    pub user_id: Uuid,
    pub vote_option: Uuid,
    pub voted_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Clone, Debug, Validate, ToSchema)]
pub struct ElectionVotesModelNew {
    pub election_id: Uuid,
    pub user_id: Uuid,
    pub vote_option: Uuid,
}

#[derive(Serialize, Deserialize, Clone, Debug, Validate, ToSchema)]
pub struct ElectionVotesModelResult {
    pub candidate: Uuid,
    pub votes: i64,
    pub votes_pct: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug, Validate, ToSchema)]
pub struct ElectionModelResult {
    pub election: ElectionModel,
    pub candidates: Vec<ElectionCandidatesModel>,
    pub votes: Vec<ElectionVotesModelResult>,
}

impl ElectionModel {
    pub fn db_count_all_matching(
        user_role: user_role_model::UserRoleModel,
        conn: &mut PgConnection,
    ) -> diesel::QueryResult<i64> {
        let mut query = ElectionModel::table().into_boxed();

        match user_role.role {
            UserRoles::Root => {}
            UserRoles::Admin | UserRoles::Resident => {
                query = query.filter(elections::community_id.eq(user_role.community_id.unwrap()));
            }
        }

        query
            .select(elections::all_columns)
            .count()
            .get_result::<i64>(conn)
    }

    pub fn db_read_all_matching_by_range(
        user_role: user_role_model::UserRoleModel,
        conn: &mut PgConnection,
        per_page: i64,
        offset: i64,
    ) -> diesel::QueryResult<Vec<ElectionModelResult>> {
        let mut query = ElectionModel::table().into_boxed();

        match user_role.role {
            UserRoles::Root => {}
            UserRoles::Admin | UserRoles::Resident => {
                query = query.filter(elections::community_id.eq(user_role.community_id.unwrap()));
            }
        }

        // Get elections
        let elections = query
            .select(elections::all_columns)
            .limit(per_page)
            .offset(offset)
            .load::<ElectionModel>(conn)?;

        let mut results = Vec::new();

        // For each election, gather candidates and votes
        for election in elections {
            // Get candidates for this election
            let candidates = ElectionCandidatesModel::table()
                .filter(election_candidates::election_id.eq(election.id))
                .load::<ElectionCandidatesModel>(conn)?;

            // Calculate votes
            let mut vote_results = Vec::new();

            // Get total votes for this election
            let total_votes = ElectionVotesModel::table()
                .filter(votes::election_id.eq(election.id))
                .count()
                .get_result::<i64>(conn)?;

            // For each candidate, calculate votes and percentage
            for candidate in &candidates {
                let candidate_votes = ElectionVotesModel::table()
                    .filter(votes::election_id.eq(election.id))
                    .filter(votes::vote_option.eq(candidate.id))
                    .count()
                    .get_result::<i64>(conn)?;

                // Calculate percentage (avoid division by zero)
                let votes_pct = if total_votes > 0 {
                    (candidate_votes * 100) / total_votes
                } else {
                    0
                };

                vote_results.push(ElectionVotesModelResult {
                    candidate: candidate.id,
                    votes: candidate_votes,
                    votes_pct,
                });
            }

            results.push(ElectionModelResult {
                election,
                candidates,
                votes: vote_results,
            });
        }

        Ok(results)
    }

    pub fn db_read_by_id_matching(
        user_role: user_role_model::UserRoleModel,
        conn: &mut PgConnection,
        id: uuid::Uuid,
    ) -> diesel::QueryResult<ElectionModelResult> {
        let election = ElectionModel::db_read_by_id(conn, id)?;

        match user_role.role {
            UserRoles::Root => {}
            UserRoles::Admin => {
                if user_role.community_id != Some(election.community_id) {
                    return Err(diesel::result::Error::NotFound);
                }
            }
            UserRoles::Resident => {
                if user_role.community_id != Some(election.community_id) {
                    return Err(diesel::result::Error::NotFound);
                }
            }
        }

        // Get candidates for this election
        let candidates = ElectionCandidatesModel::table()
            .filter(election_candidates::election_id.eq(election.id))
            .load::<ElectionCandidatesModel>(conn)?;

        // Calculate votes
        let mut vote_results = Vec::new();

        // Get total votes for this election
        let total_votes = ElectionVotesModel::table()
            .filter(votes::election_id.eq(election.id))
            .count()
            .get_result::<i64>(conn)?;

        // For each candidate, calculate votes and percentage
        for candidate in &candidates {
            let candidate_votes = ElectionVotesModel::table()
                .filter(votes::election_id.eq(election.id))
                .filter(votes::vote_option.eq(candidate.id))
                .count()
                .get_result::<i64>(conn)?;

            // Calculate percentage (avoid division by zero)
            let votes_pct = if total_votes > 0 {
                (candidate_votes * 100) / total_votes
            } else {
                0
            };

            vote_results.push(ElectionVotesModelResult {
                candidate: candidate.id,
                votes: candidate_votes,
                votes_pct,
            });
        }

        Ok(ElectionModelResult {
            election,
            candidates,
            votes: vote_results,
        })
    }

    pub fn can_user_vote(
        user_role: user_role_model::UserRoleModel,
        conn: &mut PgConnection,
        election_id: Uuid,
    ) -> diesel::QueryResult<bool> {
        // Only Residents can vote
        if user_role.role != UserRoles::Resident {
            return Ok(false);
        }

        // Get the election
        let election = ElectionModel::db_read_by_id(conn, election_id)?;

        // Check if user's community matches election's community
        if user_role.community_id != Some(election.community_id) {
            return Ok(false);
        }

        // Check if the current time is between start_date and end_date
        let current_time = chrono::Utc::now().naive_utc();
        if current_time < election.start_date || current_time > election.end_date {
            return Ok(false);
        }

        // Check if the user has already voted in this election
        let has_voted = ElectionVotesModel::table()
            .filter(votes::election_id.eq(election_id))
            .filter(votes::user_id.eq(user_role.user_id))
            .count()
            .get_result::<i64>(conn)?;

        // User can vote if they haven't voted yet
        Ok(has_voted == 0)
    }
}
