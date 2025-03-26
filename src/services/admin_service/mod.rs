pub mod get_admin;
pub mod upsert_admin;

use super::prelude::*;
type AdminListHttpResponse = HttpResponseObject<Vec<admin_model::AdminModel>>;
type AdminGetHttpResponse = HttpResponseObject<admin_model::AdminModel>;

#[derive(OpenApi)]
#[openapi(
    paths(
        get_admin::get_admins,
        get_admin::get_admin_by_id,
        upsert_admin::new_admin,
        upsert_admin::update_admin,
        upsert_admin::delete_admin,
    ),
    components(schemas(admin_model::AdminModel, admin_model::AdminModelNew))
)]
pub struct AdminApi;
