use actix_cors::Cors;
use actix_web::http::header;


pub fn cors() -> Cors {
    Cors::default()
        .allow_any_origin()
        .allow_credentials(true)
        .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
        .allowed_headers(vec![
            header::AUTHORIZATION,
            header::ACCEPT,
            header::CONTENT_TYPE,
        ])
        .max_age(3600)
}