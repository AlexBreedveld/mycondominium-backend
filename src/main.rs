mod cli;

use std::env;
use actix_web::{web, App, HttpServer};
use actix_web::middleware::Logger;
use clap::Parser;
use dotenvy::dotenv;
use log::{log, Level};
use mycondominium_backend::routes::routes::resident_route;
use crate::cli::{CliArgs, Commands};

#[tokio::main]
async fn main() {
    env_logger::init();
    dotenv().ok();
    let args = CliArgs::parse();
    
    match &args.command {
        Some(Commands::Daemon) => {
            // TODO: Implement running DB migrations before starting the server

            let http_host = env::var("SERVER_HOST").expect("SERVER_HOST must be set");
            let http_port: i32 = env::var("SERVER_PORT").expect("SERVER_PORT must be set").parse().unwrap();

            log::info!("Starting server on {}:{}", http_host, http_port);

            HttpServer::new(|| {
                App::new()
                    .wrap(Logger::default())
                    .service(resident_route())
            })
                .bind(format!("{http_host}:{http_port}")).unwrap_or_else(|_| panic!("Error binding server to: {}:{}", http_host, http_port))
                .run()
                .await.unwrap();

        },
        Some(Commands::GenerateSwagger) => {
            todo!("Implement generating OpenAPI-Swagger documentation.");
        },
        None => {
            println!("No command provided. Use --help for more information.");
        }
    }
}
