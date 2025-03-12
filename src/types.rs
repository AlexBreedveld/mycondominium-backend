use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct HttpResponseObject<T> {
    #[schema(default = false)]
    pub error: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object: Option<T>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct HttpResponseObjectEmpty {
    #[schema(default = false)]
    pub error: bool,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct HttpResponseObjectEmptyEntity {
    #[schema(default = false)]
    pub error: bool,
    pub message: String,
    pub entity_id: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct HttpResponseObjectEmptyError {
    #[schema(default = true)]
    pub error: bool,
    pub message: String,
}

#[derive(Deserialize)]
pub struct PaginationParams {
    pub(crate) page: Option<i64>,
    pub(crate) per_page: Option<i64>,
}
