use actix_web::middleware::Logger;
use actix_web::{App, HttpServer, web};
use diesel_migrations::MigrationHarness;
use mycondominium_backend::internal::scheduled_tasks::scheduled_tasks_service::scheduled_tasks_service;
use mycondominium_backend::routes::routes::*;
use mycondominium_backend::services::MyCondominiumConfig;
use mycondominium_backend::{MIGRATIONS, establish_connection_pg};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Once};
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

// Initialize logger once
static INIT: Once = Once::new();

pub fn initialize() {
    INIT.call_once(|| {
        env_logger::init();
    });
}

// =========== Common Test Client Structure ===========

struct TestClient {
    client: Client,
    base_url: String,
}

impl TestClient {
    fn new(base_url: &str) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create reqwest client");

        TestClient {
            client,
            base_url: base_url.to_string(),
        }
    }
}

fn get_test_client() -> TestClient {
    TestClient::new("http://127.0.0.1:3031")
}

// =========== Common Model Structures ===========

#[derive(Debug, Serialize, Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct LoginResponse {
    error: bool,
    message: String,
    object: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ErrorResponse {
    error: bool,
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SuccessResponse<T> {
    error: bool,
    message: String,
    object: Option<T>,
}

#[derive(Debug, Serialize, Deserialize)]
struct EntityResponse {
    error: bool,
    message: String,
    entity_id: Option<String>,
}

// =========== Admin Models ===========

#[derive(Debug, Serialize, Deserialize)]
struct AdminModel {
    id: String,
    first_name: String,
    last_name: String,
    phone: String,
    email: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct AdminModelNew {
    first_name: String,
    last_name: String,
    phone: String,
    email: String,
    password: String,
    role: String,
    community_id: Option<String>,
}

// =========== Community Models ===========

#[derive(Debug, Serialize, Deserialize)]
struct CommunityModel {
    id: String,
    name: String,
    short_name: String,
    address: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct CommunityModelNew {
    name: String,
    short_name: String,
    address: String,
}

// =========== Resident Models ===========

#[derive(Debug, Serialize, Deserialize)]
struct ResidentModel {
    id: String,
    first_name: String,
    last_name: String,
    phone: String,
    email: String,
    community_id: String,
    apartment_number: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ResidentModelNew {
    first_name: String,
    last_name: String,
    phone: String,
    email: String,
    password: String,
    community_id: String,
    apartment_number: String,
}

// =========== Authentication Functions ===========

async fn login(test_client: &TestClient, email: &str, password: &str) -> Option<String> {
    let login_req = LoginRequest {
        email: email.to_string(),
        password: password.to_string(),
    };

    let response = test_client
        .client
        .post(&format!("{}/api/auth/signin", test_client.base_url))
        .header(
            "User-agent",
            "Mozilla/5.0 (X11; Linux x86_64; rv:139.0) Gecko/20100101 Firefox/139.0",
        )
        .json(&login_req)
        .send()
        .await
        .expect("Failed to send login request");

    if response.status().is_success() {
        let login_res: LoginResponse = response
            .json()
            .await
            .expect("Failed to parse login response");
        Some(login_res.object)
    } else {
        None
    }
}

// =========== Admin Test Functions ===========

async fn test_get_admins(test_client: &TestClient, token: &str) -> Result<Vec<AdminModel>, String> {
    let response = test_client
        .client
        .get(&format!("{}/api/admin/list", test_client.base_url))
        .header("X-Auth-Token", token)
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    if response.status().is_success() {
        let response_data: SuccessResponse<Vec<AdminModel>> = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        Ok(response_data.object.unwrap_or_default())
    } else {
        let error: ErrorResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse error response: {}", e))?;

        Err(error.message)
    }
}

async fn test_get_admin_by_id(
    test_client: &TestClient,
    token: &str,
    id: &str,
) -> Result<AdminModel, String> {
    let response = test_client
        .client
        .get(&format!("{}/api/admin/get/{}", test_client.base_url, id))
        .header("X-Auth-Token", token)
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    if response.status().is_success() {
        let response_data: SuccessResponse<AdminModel> = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        Ok(response_data.object.ok_or("Admin not found in response")?)
    } else {
        let error: ErrorResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse error response: {}", e))?;

        Err(error.message)
    }
}

async fn test_create_admin(
    test_client: &TestClient,
    admin: AdminModelNew,
) -> Result<String, String> {
    let response = test_client
        .client
        .post(&format!("{}/api/admin/new", test_client.base_url))
        .json(&admin)
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    if response.status().is_success() {
        let response_data: EntityResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        Ok(response_data
            .entity_id
            .ok_or("Admin ID not found in response")?)
    } else {
        let error: ErrorResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse error response: {}", e))?;

        Err(error.message)
    }
}

async fn test_update_admin(
    test_client: &TestClient,
    token: &str,
    id: &str,
    admin: AdminModelNew,
) -> Result<(), String> {
    let response = test_client
        .client
        .put(&format!("{}/api/admin/update/{}", test_client.base_url, id))
        .header("X-Auth-Token", token)
        .json(&admin)
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    if response.status().is_success() {
        Ok(())
    } else {
        let error: ErrorResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse error response: {}", e))?;

        Err(error.message)
    }
}

async fn test_delete_admin(test_client: &TestClient, token: &str, id: &str) -> Result<(), String> {
    let response = test_client
        .client
        .delete(&format!("{}/api/admin/delete/{}", test_client.base_url, id))
        .header("X-Auth-Token", token)
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    if response.status().is_success() {
        Ok(())
    } else {
        let error: ErrorResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse error response: {}", e))?;

        Err(error.message)
    }
}

// =========== Community Test Functions ===========

async fn test_get_communities(
    test_client: &TestClient,
    token: &str,
) -> Result<Vec<CommunityModel>, String> {
    let response = test_client
        .client
        .get(&format!("{}/api/community/list", test_client.base_url))
        .header("X-Auth-Token", token)
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    if response.status().is_success() {
        let response_data: SuccessResponse<Vec<CommunityModel>> = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        Ok(response_data.object.unwrap_or_default())
    } else {
        let error: ErrorResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse error response: {}", e))?;

        Err(error.message)
    }
}

async fn test_get_community_by_id(
    test_client: &TestClient,
    token: &str,
    id: &str,
) -> Result<CommunityModel, String> {
    let response = test_client
        .client
        .get(&format!(
            "{}/api/community/get/{}",
            test_client.base_url, id
        ))
        .header("X-Auth-Token", token)
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    if response.status().is_success() {
        let response_data: SuccessResponse<CommunityModel> = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        Ok(response_data
            .object
            .ok_or("Community not found in response")?)
    } else {
        let error: ErrorResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse error response: {}", e))?;

        Err(error.message)
    }
}

async fn test_create_community(
    test_client: &TestClient,
    token: &str,
    community: CommunityModelNew,
) -> Result<String, String> {
    let response = test_client
        .client
        .post(&format!("{}/api/community/new", test_client.base_url))
        .header("X-Auth-Token", token)
        .json(&community)
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    if response.status().is_success() {
        let response_data: EntityResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        Ok(response_data
            .entity_id
            .ok_or("Community ID not found in response")?)
    } else {
        let error: ErrorResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse error response: {}", e))?;

        Err(error.message)
    }
}

async fn test_update_community(
    test_client: &TestClient,
    token: &str,
    id: &str,
    community: CommunityModelNew,
) -> Result<(), String> {
    let response = test_client
        .client
        .put(&format!(
            "{}/api/community/update/{}",
            test_client.base_url, id
        ))
        .header("X-Auth-Token", token)
        .json(&community)
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    if response.status().is_success() {
        Ok(())
    } else {
        let error: ErrorResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse error response: {}", e))?;

        Err(error.message)
    }
}

async fn test_delete_community(
    test_client: &TestClient,
    token: &str,
    id: &str,
) -> Result<(), String> {
    let response = test_client
        .client
        .delete(&format!(
            "{}/api/community/delete/{}",
            test_client.base_url, id
        ))
        .header("X-Auth-Token", token)
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    if response.status().is_success() {
        Ok(())
    } else {
        let error: ErrorResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse error response: {}", e))?;

        Err(error.message)
    }
}

// =========== Resident Test Functions ===========

async fn test_get_residents(
    test_client: &TestClient,
    token: &str,
) -> Result<Vec<ResidentModel>, String> {
    let response = test_client
        .client
        .get(&format!("{}/api/resident/list", test_client.base_url))
        .header("X-Auth-Token", token)
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    if response.status().is_success() {
        let response_data: SuccessResponse<Vec<ResidentModel>> = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        Ok(response_data.object.unwrap_or_default())
    } else {
        let error: ErrorResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse error response: {}", e))?;

        Err(error.message)
    }
}

async fn test_get_resident_by_id(
    test_client: &TestClient,
    token: &str,
    id: &str,
) -> Result<ResidentModel, String> {
    let response = test_client
        .client
        .get(&format!("{}/api/resident/get/{}", test_client.base_url, id))
        .header("X-Auth-Token", token)
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    if response.status().is_success() {
        let response_data: SuccessResponse<ResidentModel> = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        Ok(response_data
            .object
            .ok_or("Resident not found in response")?)
    } else {
        let error: ErrorResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse error response: {}", e))?;

        Err(error.message)
    }
}

async fn test_create_resident(
    test_client: &TestClient,
    token: &str,
    resident: ResidentModelNew,
) -> Result<String, String> {
    let response = test_client
        .client
        .post(&format!("{}/api/resident/new", test_client.base_url))
        .header("X-Auth-Token", token)
        .json(&resident)
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    if response.status().is_success() {
        let response_data: EntityResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        Ok(response_data
            .entity_id
            .ok_or("Resident ID not found in response")?)
    } else {
        let error: ErrorResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse error response: {}", e))?;

        Err(error.message)
    }
}

async fn test_update_resident(
    test_client: &TestClient,
    token: &str,
    id: &str,
    resident: ResidentModelNew,
) -> Result<(), String> {
    let response = test_client
        .client
        .put(&format!(
            "{}/api/resident/update/{}",
            test_client.base_url, id
        ))
        .header("X-Auth-Token", token)
        .json(&resident)
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    if response.status().is_success() {
        Ok(())
    } else {
        let error: ErrorResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse error response: {}", e))?;

        Err(error.message)
    }
}

async fn test_delete_resident(
    test_client: &TestClient,
    token: &str,
    id: &str,
) -> Result<(), String> {
    let response = test_client
        .client
        .delete(&format!(
            "{}/api/resident/delete/{}",
            test_client.base_url, id
        ))
        .header("X-Auth-Token", token)
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    if response.status().is_success() {
        Ok(())
    } else {
        let error: ErrorResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse error response: {}", e))?;

        Err(error.message)
    }
}

// =========== Test Implementations ===========

async fn test_admin_crud(test_client: &TestClient) -> Result<(), String> {
    println!("Running Admin CRUD tests...");

    // Test creating a new admin
    let new_admin = AdminModelNew {
        first_name: "Test".to_string(),
        last_name: "Admin".to_string(),
        phone: "+1234567890".to_string(),
        email: "test-admin@example.com".to_string(),
        password: "password123".to_string(),
        role: "Root".to_string(),
        community_id: None, // Use a valid community ID
    };

    let admin_id = test_create_admin(test_client, new_admin.clone()).await?;
    println!("Created admin with ID: {}", admin_id);

    // First login as admin
    let token = login(test_client, "test-admin@example.com", "password123")
        .await
        .ok_or("Failed to login")?;

    // Test getting the created admin
    let admin = test_get_admin_by_id(test_client, &token, &admin_id).await?;

    assert_eq!(admin.first_name, new_admin.first_name);
    assert_eq!(admin.last_name, new_admin.last_name);
    assert_eq!(admin.phone, new_admin.phone);
    assert_eq!(admin.email, new_admin.email);

    // Test updating the admin
    let updated_admin = AdminModelNew {
        first_name: "Updated".to_string(),
        last_name: "Admin".to_string(),
        phone: "+0987654321".to_string(),
        email: new_admin.email.clone(),
        password: "password123".to_string(),
        role: "Admin".to_string(),
        community_id: new_admin.community_id.clone(),
    };

    test_update_admin(test_client, &token, &admin_id, updated_admin.clone()).await?;
    println!("Updated admin with ID: {}", admin_id);

    // Test getting the updated admin
    let admin = test_get_admin_by_id(test_client, &token, &admin_id).await?;

    assert_eq!(admin.first_name, updated_admin.first_name);
    assert_eq!(admin.last_name, updated_admin.last_name);
    assert_eq!(admin.phone, updated_admin.phone);

    // Test deleting the admin
    test_delete_admin(test_client, &token, &admin_id).await?;
    println!("Deleted admin with ID: {}", admin_id);

    // Test that admin no longer exists
    let result = test_get_admin_by_id(test_client, &token, &admin_id).await;
    assert!(result.is_err());

    println!("All Admin CRUD tests passed!");
    Ok(())
}

async fn test_community_crud(test_client: &TestClient) -> Result<String, String> {
    println!("Running Community CRUD tests...");

    // First login as admin
    let token = login(test_client, "admin@example.com", "password123")
        .await
        .ok_or("Failed to login")?;

    // Test creating a new community
    let new_community = CommunityModelNew {
        name: format!("Test Community {}", Uuid::new_v4()),
        short_name: "TEST".to_string(),
        address: "123 Test Street, Test City".to_string(),
    };

    let community_id = test_create_community(test_client, &token, new_community.clone()).await?;
    println!("Created community with ID: {}", community_id);

    // Test getting the created community
    let community = test_get_community_by_id(test_client, &token, &community_id).await?;

    assert_eq!(community.name, new_community.name);
    assert_eq!(community.short_name, new_community.short_name);
    assert_eq!(community.address, new_community.address);

    // Test updating the community
    let updated_community = CommunityModelNew {
        name: format!("Updated Community {}", Uuid::new_v4()),
        short_name: "UPDT".to_string(),
        address: "456 Update Avenue, New City".to_string(),
    };

    test_update_community(
        test_client,
        &token,
        &community_id,
        updated_community.clone(),
    )
    .await?;
    println!("Updated community with ID: {}", community_id);

    // Test getting the updated community
    let community = test_get_community_by_id(test_client, &token, &community_id).await?;

    assert_eq!(community.name, updated_community.name);
    assert_eq!(community.short_name, updated_community.short_name);
    assert_eq!(community.address, updated_community.address);

    println!("Community CRUD tests passed!");

    // Return the community ID for resident tests
    Ok(community_id)
}

async fn test_resident_crud(test_client: &TestClient, community_id: &str) -> Result<(), String> {
    println!("Running Resident CRUD tests...");

    // First login as admin
    let token = login(test_client, "admin@example.com", "password123")
        .await
        .ok_or("Failed to login")?;

    // Test creating a new resident
    let new_resident = ResidentModelNew {
        first_name: "Test".to_string(),
        last_name: "Resident".to_string(),
        phone: "+1234567890".to_string(),
        email: format!("test-resident-{}@example.com", Uuid::new_v4()),
        password: "password123".to_string(),
        community_id: community_id.to_string(),
        apartment_number: "A-101".to_string(),
    };

    let resident_id = test_create_resident(test_client, &token, new_resident.clone()).await?;
    println!("Created resident with ID: {}", resident_id);

    // Test getting the created resident
    let resident = test_get_resident_by_id(test_client, &token, &resident_id).await?;

    assert_eq!(resident.first_name, new_resident.first_name);
    assert_eq!(resident.last_name, new_resident.last_name);
    assert_eq!(resident.phone, new_resident.phone);
    assert_eq!(resident.email, new_resident.email);
    assert_eq!(resident.community_id, new_resident.community_id);
    assert_eq!(resident.apartment_number, new_resident.apartment_number);

    // Test updating the resident
    let updated_resident = ResidentModelNew {
        first_name: "Updated".to_string(),
        last_name: "Resident".to_string(),
        phone: "+0987654321".to_string(),
        email: new_resident.email.clone(),
        password: "password123".to_string(),
        community_id: new_resident.community_id.clone(),
        apartment_number: "B-202".to_string(),
    };

    test_update_resident(test_client, &token, &resident_id, updated_resident.clone()).await?;
    println!("Updated resident with ID: {}", resident_id);

    // Test getting the updated resident
    let resident = test_get_resident_by_id(test_client, &token, &resident_id).await?;

    assert_eq!(resident.first_name, updated_resident.first_name);
    assert_eq!(resident.last_name, updated_resident.last_name);
    assert_eq!(resident.phone, updated_resident.phone);
    assert_eq!(resident.apartment_number, updated_resident.apartment_number);

    // Test deleting the resident
    test_delete_resident(test_client, &token, &resident_id).await?;
    println!("Deleted resident with ID: {}", resident_id);

    // Test that resident no longer exists
    let result = test_get_resident_by_id(test_client, &token, &resident_id).await;
    assert!(result.is_err());

    // Clean up by removing the community we created
    test_delete_community(test_client, &token, community_id).await?;
    println!("Deleted community with ID: {}", community_id);

    println!("All Resident CRUD tests passed!");
    Ok(())
}

// =========== Server Setup & Main Test ===========

async fn start_server() {
    let config_path = &PathBuf::from("./config-tests.yaml");

    if !config_path.exists() || !config_path.is_file() {
        log::error!("Configuration file not found at: {}", config_path.display());
        panic!("Configuration file not found at: {}", config_path.display());
    }

    let conf_str = match std::fs::read_to_string(config_path) {
        Ok(dat) => dat,
        Err(e) => {
            log::error!("Failed to read configuration file: {}", e);
            panic!("Failed to read configuration file: {}", e);
        }
    };

    let conf: MyCondominiumConfig = match serde_yaml::from_str(&conf_str) {
        Ok(dat) => dat,
        Err(e) => {
            log::error!("Failed to parse configuration file: {}", e);
            panic!("Failed to parse configuration file: {}", e);
        }
    };

    let conf: Arc<MyCondominiumConfig> = Arc::new(conf);

    let conn = &mut establish_connection_pg(&conf);
    conn.run_pending_migrations(MIGRATIONS)
        .expect("Failed to run database migrations");

    tokio::spawn(scheduled_tasks_service(conf.clone()));

    let conf_clone = conf.clone();

    log::info!(
        "Starting server on {}:{}",
        &conf_clone.server.host,
        &conf_clone.server.port
    );
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(conf_clone.clone()))
            .service(resident_route())
            .service(admin_route())
            .service(auth_route())
            .service(community_route())
            .service(vehicle_route())
            .service(maintenance_schedule_route())
            .service(parcel_route())
            .service(common_area_route())
            .service(reservation_route())
            .service(invoice_route())
    })
    .bind(format!("{}:{}", conf.server.host, conf.server.port))
    .unwrap_or_else(|_| {
        panic!(
            "Error binding server to: {}:{}",
            conf.server.host, conf.server.port
        )
    })
    .run()
    .await
    .unwrap();
}

//#[tokio::test]
async fn test_main() {
    Command::new("sudo")
        .args([
            "-u",
            "postgres",
            "psql",
            "-c",
            "DROP DATABASE mycondominium_tests;",
        ])
        .output()
        .expect("Failed to execute command");

    sleep(Duration::from_secs(1)).await;

    Command::new("sudo")
        .args([
            "-u",
            "postgres",
            "psql",
            "-c",
            "CREATE DATABASE mycondominium_tests;",
        ])
        .output()
        .expect("Failed to execute command");

    sleep(Duration::from_secs(1)).await;

    initialize();

    // Start the server
    let server_handle = tokio::spawn(start_server());

    println!("Waiting for server to start...");
    sleep(Duration::from_secs(1)).await;
    println!("Server started, running tests...");

    let test_client = get_test_client();

    // Run admin tests
    match test_admin_crud(&test_client).await {
        Ok(_) => println!("Admin tests completed successfully"),
        Err(e) => panic!("Admin tests failed: {}", e),
    }

    // Run community tests (returns the community ID for resident tests)
    let community_id = match test_community_crud(&test_client).await {
        Ok(id) => {
            println!("Community tests completed successfully");
            id
        }
        Err(e) => panic!("Community tests failed: {}", e),
    };

    // Run resident tests with the community ID
    match test_resident_crud(&test_client, &community_id).await {
        Ok(_) => println!("Resident tests completed successfully"),
        Err(e) => panic!("Resident tests failed: {}", e),
    }

    // Abort the server after tests complete
    server_handle.abort();
    println!("All tests completed successfully!");
}
