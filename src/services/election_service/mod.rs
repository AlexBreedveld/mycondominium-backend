pub mod get_election;
pub mod get_vote;
pub mod upsert_candidate;
pub mod upsert_election;
pub mod upsert_vote;

use super::prelude::*;
type ElectionListHttpResponse = HttpResponseObject<Vec<election_model::ElectionModelResult>>;
type ElectionGetHttpResponse = HttpResponseObject<election_model::ElectionModelResult>;

#[derive(OpenApi)]
#[openapi(
    paths(
        get_election::get_elections,
        get_election::get_election_by_id,
        upsert_election::new_election,
        upsert_election::update_election,
        upsert_election::delete_election,
        upsert_candidate::new_election_candidate,
        upsert_candidate::update_election_candidate,
        upsert_candidate::delete_election_candidate,
        upsert_vote::new_election_vote,
        get_vote::get_can_vote,
    ),
    components(schemas(
        election_model::ElectionModel,
        election_model::ElectionModelNew,
        election_model::ElectionCandidatesModel,
        election_model::ElectionCandidatesModelNew,
        election_model::ElectionVotesModel,
        election_model::ElectionVotesModelNew,
        election_model::ElectionVotesModelResult,
        election_model::ElectionModelResult,
    ))
)]
pub struct ElectionApi;
