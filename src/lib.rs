pub mod internal;
pub mod models;
pub mod routes;
pub mod schema;
pub mod services;
pub mod types;
pub mod utilities;

use diesel_migrations::{EmbeddedMigrations, embed_migrations};
use crate::internal::config::model::ConfigRabbitmq;
use std::env;
use crate::internal::config::model::MyCondominiumConfig;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

pub fn establish_connection_pg(conf: &MyCondominiumConfig) -> diesel::PgConnection {
    use diesel::prelude::*;
    
    PgConnection::establish(&conf.database.url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", &conf.database.url))
}

pub async fn establish_connection_minio()
-> Result<minio::s3::client::Client, Box<dyn std::error::Error + Send + Sync>> {
    use minio::s3::args::{BucketExistsArgs, MakeBucketArgs};
    use minio::s3::client::ClientBuilder;

    let minio_url = env::var("MINIO_URL").expect("MINIO_URL must be set");
    let minio_bucket_name = env::var("MINIO_BUCKET_NAME").expect("MINIO_BUCKET_NAME must be set");
    let minio_access_key = env::var("MINIO_ACCESS_KEY").expect("MINIO_ACCESS_KEY must be set");
    let minio_secret_key = env::var("MINIO_SECRET_KEY").expect("MINIO_SECRET_KEY must be set");

    let base_url = &minio_url.parse::<minio::s3::http::BaseUrl>()?;

    let static_provider =
        minio::s3::creds::StaticProvider::new(&minio_access_key, &minio_secret_key, None);

    let client = ClientBuilder::new(base_url.clone())
        .provider(Some(Box::new(static_provider)))
        .build()?;

    let exists: bool = client
        .bucket_exists(&BucketExistsArgs::new(&minio_bucket_name).unwrap())
        .await?;

    if !exists {
        client
            .make_bucket(&MakeBucketArgs::new(&minio_bucket_name).unwrap())
            .await
            .unwrap();
    };

    Ok(client)
}

pub async fn establish_connection_rabbitmq(config: &ConfigRabbitmq) -> Result<lapin::Connection, lapin::Error> {
    let addr = format!("amqp://{}:{}@{}:{}", config.username, config.password, config.host, config.port);
    lapin::Connection::connect(&addr, lapin::ConnectionProperties::default()).await
}