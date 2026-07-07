use crate::logging::LogLevel;
use crate::provider::ProviderArgs;
use crate::secrets::Secret;
use clap::{Args, Subcommand};
#[derive(Args, Debug)]
pub struct ComposeArgs {
    /// Compose Project Name
    #[arg(long = "project-name", env = "COMPOSE_PROJECT_NAME")]
    pub project_name: String,

    /// Docker Compose provider API command
    #[command(subcommand)]
    pub cmd: ComposeCommand,
}

#[derive(Args, Debug)]
pub struct UpArgs {
    /// Provider configuration
    #[command(flatten)]
    pub provider: ProviderArgs,

    /// Files containing environment variables which may contain secret references
    #[arg(
        long,
        env = "LOCKET_ENV_FILE",
        value_name = "/path/to/.env",
        alias = "env_file",
        value_delimiter = ',',
        hide_env_values = true,
        help_heading = None,
        value_parser = crate::path::parse_secret_path,
        action = clap::ArgAction::Append,
    )]
    pub env_file: Vec<Secret>,

    /// Environment variable overrides which may contain secret references
    #[arg(
        long,
        short = 'e',
        env = "LOCKET_ENV",
        value_name = "KEY=VAL, KEY=@FILE or /path/to/.env",
        value_delimiter = ',',
        hide_env_values = true,
        help_heading = None,
        action = clap::ArgAction::Append,
    )]
    pub env: Vec<Secret>,

    /// Log level
    #[arg(long, env = "LOCKET_LOG_LEVEL", value_enum, default_value_t = LogLevel::Debug)]
    pub log_level: LogLevel,

    /// Service name from Docker Compose
    #[arg(help_heading = None)]
    pub service: String,
}

/// Options passed by Docker Compose when tearing down a project.
///
/// Docker Compose invokes the provider `down` command with the same
/// options as `up`, so they must be accepted, but they are unused
/// because secrets are not persisted.
#[derive(Args, Debug)]
pub struct DownArgs {
    #[arg(trailing_var_arg = true, allow_hyphen_values = true, hide = true)]
    pub options: Vec<String>,
}

#[derive(Subcommand, Debug)]
pub enum ComposeCommand {
    /// Injects secrets into a Docker Compose service environment with `docker compose up`
    Up(Box<UpArgs>),
    /// Handler for Docker Compose `down`, but no-op because secrets are not persisted
    Down(DownArgs),
    /// Handler for Docker Compose `metadata` command so that docker can query plugin capabilities
    Metadata,
}

#[cfg(test)]
mod tests {
    use crate::cmd::Cli;
    use clap::Parser;

    /// Docker Compose invokes the provider `down` command with the same
    /// options as `up`, so they must parse even though `down` ignores them.
    #[test]
    fn test_down_accepts_provider_options() {
        Cli::try_parse_from([
            "locket",
            "compose",
            "--project-name",
            "demo",
            "down",
            "--provider",
            "bao",
            "--bao-url",
            "https://bao.example",
            "--raw-env=true",
            "--env",
            "KEY={{bao://secret/app/key}}",
            "service",
        ])
        .expect("compose down must accept the same options as up");
    }

    #[test]
    fn test_down_accepts_no_options() {
        Cli::try_parse_from(["locket", "compose", "--project-name", "demo", "down"])
            .expect("compose down must parse without options");
    }
}
