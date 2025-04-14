use std::path::PathBuf;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run the Web server.
    Daemon(DaemonArgs),
    /// Generate OpenAPI-Swagger documentation.
    GenerateSwagger,
    /// Serve only OpenAPI-Swagger documentation.
    ServeSwagger,
}

#[derive(Parser)]
#[clap(rename_all = "kebab-case")]
pub struct DaemonArgs {
    #[arg(short, long, help = "Path to the configuration file")]
    pub config_path: Option<PathBuf>,
}