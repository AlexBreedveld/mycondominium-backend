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
    Daemon,
    /// Generate OpenAPI-Swagger documentation.
    GenerateSwagger,
    /// Serve only OpenAPI-Swagger documentation.
    ServeSwagger,
}
