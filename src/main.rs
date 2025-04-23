mod cli;

use crate::cli::{CliArgs, Commands};
use actix_web::middleware::Logger;
use actix_web::{App, HttpResponse, HttpServer, web};
use clap::Parser;
use diesel_migrations::MigrationHarness;
use dotenvy::dotenv;
use mycondominium_backend::routes::routes::*;
use mycondominium_backend::services::ApiDoc;
use mycondominium_backend::{MIGRATIONS, establish_connection_pg};
use std::env;
use std::path::PathBuf;
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use mycondominium_backend::internal::config::model::MyCondominiumConfig;

#[tokio::main]
async fn main() {
    env_logger::init();
    dotenv().ok();
    let args = CliArgs::parse();
    let conf: MyCondominiumConfig;
    match &args.command {
        Some(Commands::Daemon(daemon_args)) => {
            let config_path = match &daemon_args.config_path {
                Some(path) => path,
                None => { &PathBuf::from("config.yaml") },
            };

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

            log::info!("Starting server on {}:{}", &conf.server.host, &conf.server.port);
            let conf_clone = conf.clone();
            HttpServer::new(move || {
                App::new()
                    .wrap(Logger::default())
                    .app_data(web::Data::new(conf_clone.clone()))
                    .service(resident_route())
                    .service(admin_route())
                    .service(auth_route())
                    .service(community_route())
                    .service(vehicle_route())
                    .service(
                        SwaggerUi::new("/docs-v1/{_:.*}")
                            .url("/api-docs/openapi.json", ApiDoc::openapi()),
                    )
                    .route(
                        "/docs-v1",
                        web::get().to(|_req: actix_web::HttpRequest| async {
                            HttpResponse::Found()
                                .append_header(("Location", "/docs-v1/"))
                                .finish()
                        }),
                    )
            })
            .bind(format!("{}:{}", conf.server.host, conf.server.port))
            .unwrap_or_else(|_| panic!("Error binding server to: {}:{}", conf.server.host, conf.server.port))
            .run()
            .await
            .unwrap();
        }
        Some(Commands::GenerateSwagger) => {
            println!("{}", ApiDoc::openapi().to_pretty_json().unwrap());
        }
        Some(Commands::ServeSwagger) => {
            let http_host = env::var("SERVER_HOST").expect("SERVER_HOST must be set");
            let http_port: i32 = env::var("SERVER_PORT")
                .expect("SERVER_PORT must be set")
                .parse()
                .unwrap();

            log::info!("Starting server on {}:{}", http_host, http_port);

            HttpServer::new(|| {
                App::new().wrap(Logger::default()).service(
                    SwaggerUi::new("/docs-v1/{_:.*}")
                        .url("/api-docs/openapi.json", ApiDoc::openapi()),
                )
            })
            .bind(format!("{http_host}:{http_port}"))
            .unwrap_or_else(|_| panic!("Error binding server to: {}:{}", http_host, http_port))
            .run()
            .await
            .unwrap();
        }
        None => {
            println!("No command provided. Use --help for more information.");
        }
    }
}
