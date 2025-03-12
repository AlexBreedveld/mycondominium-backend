use actix_web::web;

pub fn resident_route() -> actix_web::Scope {
    web::scope("/api/resident")
        .route("/list", web::get().to(crate::services::resident_service::get_resident::get_residents))
        .route("/get/{id}", web::get().to(crate::services::resident_service::get_resident::get_resident_by_id))
        .route("/new", web::post().to(crate::services::resident_service::upsert_resident::new_resident))
        .route("/update/{id}", web::put().to(crate::services::resident_service::upsert_resident::update_resident))
        .route("/delete/{id}", web::delete().to(crate::services::resident_service::upsert_resident::delete_resident))
}
