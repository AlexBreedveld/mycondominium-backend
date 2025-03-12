mod cli;

use crate::cli::{CliArgs, Commands};
use actix_web::middleware::Logger;
use actix_web::{App, HttpServer, web};
use clap::Parser;
use dotenvy::dotenv;
use log::{Level, log};
use mycondominium_backend::routes::routes::resident_route;
use mycondominium_backend::services::ApiDoc;
use std::env;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() {
    env_logger::init();
    dotenv().ok();
    let args = CliArgs::parse();

    match &args.command {
        Some(Commands::Daemon) => {
            // TODO: Implement running DB migrations before starting the server

            let http_host = env::var("SERVER_HOST").expect("SERVER_HOST must be set");
            let http_port: i32 = env::var("SERVER_PORT")
                .expect("SERVER_PORT must be set")
                .parse()
                .unwrap();

            log::info!("Starting server on {}:{}", http_host, http_port);

            HttpServer::new(|| {
                App::new()
                    .wrap(Logger::default())
                    .service(resident_route())
                    .service(
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
