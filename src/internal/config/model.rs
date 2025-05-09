use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MyCondominiumConfig {
    #[serde(rename = "database")]
    pub database: ConfigDatabase,

    #[serde(rename = "smtp")]
    pub smtp: ConfigSmtp,

    #[serde(rename = "minio")]
    pub minio: ConfigMinio,

    #[serde(rename = "server")]
    pub server: ConfigServer,

    #[serde(rename = "auth")]
    pub auth: ConfigAuth,

    #[serde(rename = "rabbitmq")]
    pub rabbitmq: ConfigRabbitmq,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigDatabase {
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigSmtp {
    pub host: String,
    pub port: String,
    pub username: String,
    pub password: String,
    pub from: String,
    pub base_url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigMinio {
    pub url: String,
    pub bucket_name: String,
    pub access_key: String,
    pub secret_key: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigServer {
    pub host: String,
    pub port: String,
    pub cors_permissive: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigAuth {
    pub token_expiration_days: u32,
    #[serde(rename = "token_secret_key")]
    pub token_secret_key: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigRabbitmq {
    pub host: String,
    pub port: String,
    pub username: String,
    pub password: String,
}
