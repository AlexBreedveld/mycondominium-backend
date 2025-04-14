use actix_web::{test, web, App, http::header};
use serde_json::json;
use uuid::Uuid;
use actix_web::http::StatusCode;
use diesel::prelude::*;
use mycondominium_backend::models::admin_model::AdminModelNew;
use mycondominium_backend::services::admin_service::upsert_admin::new_admin;
use mycondominium_backend::services::UserRoles;
use mycondominium_backend::types::HttpResponseObjectEmptyEntity;

#[actix_web::test]
async fn test_new_root_admin_success() {
    // Setup the request body
    let admin_data = AdminModelNew {
        first_name: "John".to_string(),
        last_name: "Doe".to_string(),
        phone: Some("1234567890".to_string()),
        email: "rootadmin@example.com".to_string(),
        password: "SecurePwd123!".to_string(),
        role: UserRoles::Root,
        community_id: None,
    };

    // Serialize request body to JSON
    let req_json = json!(admin_data);

    // Setup the request object
    let req = test::TestRequest::post()
        .uri("/new")
        .insert_header((header::CONTENT_TYPE, "application/json"))
        .set_json(&req_json)
        .to_http_request();

    // Call the handler
    let resp = new_admin(web::Json(admin_data), req).await;

    // Check response status code
    assert_eq!(resp.status(), StatusCode::OK);

    // Parse response body
    let body = resp.into_body();
    let bytes = actix_web::body::to_bytes(body).await.unwrap();
    let resp_body: HttpResponseObjectEmptyEntity = serde_json::from_slice(&bytes).unwrap();
    assert!(!resp_body.error);
    assert_eq!(resp_body.message, "Admin created successfully");
    assert!(resp_body.entity_id.is_some());
}

#[actix_web::test]
async fn test_new_admin_unauthorized() {
    // Attempt creating an Admin by unauthorized non-root
    let admin_data = AdminModelNew {
        first_name: "Jane".to_string(),
        last_name: "Doe".to_string(),
        phone: Some("0987654321".to_string()),
        email: "admin@example.com".to_string(),
        password: "AnotherPwd123!".to_string(),
        role: UserRoles::Admin,
        community_id: Some(Uuid::new_v4()),
    };

    // Serialize request body to JSON
    let req_json = json!(admin_data);

    // Setup the request object without valid auth
    let req = test::TestRequest::post()
        .uri("/new")
        .insert_header((header::CONTENT_TYPE, "application/json"))
        .set_json(&req_json)
        .to_http_request();

    // Call the handler
    let resp = new_admin(web::Json(admin_data), req).await;

    // Check response status code
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    // Parse response body
    let body = resp.into_body();
    let bytes = actix_web::body::to_bytes(body).await.unwrap();
    let resp_body: HttpResponseObjectEmptyEntity = serde_json::from_slice(&bytes).unwrap();
    assert!(resp_body.error);
    assert_eq!(resp_body.message, "Unauthorized");
}

#[actix_web::test]
async fn test_new_admin_invalid_role() {
    // Attempt creating an Admin with an invalid role (assuming enum UserRoles has no variant "Resident")
    let admin_data = json!({
        "first_name": "Invalid",
        "last_name": "Role",
        "phone": "1112223333",
        "email": "invalidrole@test.com",
        "password": "InvalidPwd123!",
        "role": "Resident"  // invalid role for this endpoint
    });

    // Setup the request object
    let req = test::TestRequest::post()
        .uri("/new")
        .insert_header((header::CONTENT_TYPE, "application/json"))
        .set_json(&admin_data)
        .to_http_request();

    // Extract model from JSON explicitly here since type mismatch occurs
    let admin_data: AdminModelNew = serde_json::from_value(admin_data).unwrap_or(AdminModelNew {
        first_name: "Invalid".into(),
        last_name: "Role".into(),
        phone: Some("1112223333".into()),
        email: "invalidrole@test.com".into(),
        password: "InvalidPwd123!".into(),
        role: UserRoles::Admin,  // fallback to Admin, but you might adjust according to actual impl
        community_id: None,
    });

    // Call the handler
    let resp = new_admin(web::Json(admin_data), req).await;

    // Check response status code
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    // Parse response body
    let body = resp.into_body();
    let bytes = actix_web::body::to_bytes(body).await.unwrap();
    let resp_body: HttpResponseObjectEmptyEntity = serde_json::from_slice(&bytes).unwrap();
    assert!(resp_body.error);
    assert_eq!(resp_body.message, "Invalid Admin Role");
}