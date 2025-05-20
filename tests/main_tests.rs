use actix_web::middleware::Logger;
use actix_web::{App, HttpServer, web};
use diesel_migrations::MigrationHarness;
use mycondominium_backend::internal::scheduled_tasks::scheduled_tasks_service::scheduled_tasks_service;
use mycondominium_backend::routes::routes::*;
use mycondominium_backend::services::MyCondominiumConfig;
use mycondominium_backend::{MIGRATIONS, establish_connection_pg};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_main() {
    let server_handle = tokio::spawn(start_server());

    println!("Waiting for server to start...");

    sleep(Duration::from_secs(3)).await;

    println!("Server started, running tests...");

    // TODO: Add tests here

    server_handle.abort();
}

async fn start_server() {
    let config_path = &PathBuf::from("config-tests.yaml");

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
