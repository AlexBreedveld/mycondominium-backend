use actix_web::web;

pub fn resident_route() -> actix_web::Scope {
    web::scope("/resident")
        .route("/list", web::get().to(crate::services::resident_service::get_resident::get_residents))
        .route("/get/{id}", web::get().to(crate::services::resident_service::get_resident::get_resident_by_id))
}
