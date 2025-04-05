pub mod get_community;
pub mod upsert_community;

use super::prelude::*;
type CommunityListHttpResponse = HttpResponseObject<Vec<community_model::CommunityModel>>;
type CommunityGetHttpResponse = HttpResponseObject<community_model::CommunityModel>;

#[derive(OpenApi)]
#[openapi(
    paths(
        get_community::get_communities,
        get_community::get_community_by_id,
        upsert_community::new_community,
        upsert_community::update_community,
        upsert_community::delete_community,
    ),
    components(schemas(community_model::CommunityModel, community_model::CommunityModelNew))
)]
pub struct CommunityApi;
