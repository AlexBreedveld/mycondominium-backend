pub mod internal;
pub mod models;
pub mod routes;
pub mod schema;
pub mod services;
pub mod types;
pub mod utilities;

use crate::internal::config::model::ConfigRabbitmq;
use crate::internal::config::model::MyCondominiumConfig;
use diesel_migrations::{EmbeddedMigrations, embed_migrations};
use std::env;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

pub fn establish_connection_pg(conf: &MyCondominiumConfig) -> diesel::PgConnection {
    use diesel::prelude::*;

    PgConnection::establish(&conf.database.url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", &conf.database.url))
}

pub async fn establish_connection_rabbitmq(
    config: &ConfigRabbitmq,
) -> Result<lapin::Connection, lapin::Error> {
    let addr = format!(
        "amqp://{}:{}@{}:{}",
        config.username, config.password, config.host, config.port
    );
    lapin::Connection::connect(&addr, lapin::ConnectionProperties::default()).await
}
