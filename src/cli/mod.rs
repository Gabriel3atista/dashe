pub mod commands;
pub mod install;

use anyhow::Result;
use clap::{Parser, Subcommand};

use commands::*;

#[derive(Parser)]
#[command(
    name = "dashe",
    about = "⚡ Dashe — Modern terminal customizer for Linux/Bash",
    version,
    long_about = None,
    propagate_version = true,
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize Dashe with interactive setup wizard
    Init,

    /// Render the current prompt (use in PROMPT_COMMAND)
    Prompt,

    /// Manage aliases
    Alias {
        #[command(subcommand)]
        action: AliasCommands,
    },

    /// Manage themes and colors
    Theme {
        #[command(subcommand)]
        action: ThemeCommands,
    },

    /// Cloud sync and authentication
    Sync {
        #[command(subcommand)]
        action: SyncCommands,
    },

    /// Show help and usage examples
    Help {
        /// Topic to get help on (alias, theme, sync, git, prompt)
        topic: Option<String>,
    },

    /// Diagnose environment and troubleshoot issues
    Doctor,

    /// Run a startup performance benchmark
    Bench,

    /// Uninstall Dashe — removes binary and config files
    Uninstall {
        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },
}

#[derive(Subcommand)]
pub enum AliasCommands {
    /// Add a new alias
    Add {
        /// Alias name
        key: String,
        /// Command to run
        command: String,
        /// Optional category
        #[arg(short, long)]
        category: Option<String>,
    },
    /// Remove an alias
    Remove {
        /// Alias name
        key: String,
    },
    /// List all aliases
    List {
        /// Filter by category
        #[arg(short, long)]
        category: Option<String>,
    },
    /// Export aliases to a file
    Export {
        /// Output file path
        #[arg(short, long, default_value = "dashe-aliases.toml")]
        output: String,
    },
    /// Import aliases from a file
    Import {
        /// Input file path
        file: String,
    },
}

#[derive(Subcommand)]
pub enum ThemeCommands {
    /// List available themes
    List,
    /// Apply a theme
    Set {
        /// Theme name (p10k, starship, minimal, retro, ocean)
        name: String,
    },
    /// Show current theme
    Current,
    /// Export current profile
    Export {
        #[arg(short, long, default_value = "dashe-theme.toml")]
        output: String,
    },
    /// Import a theme from file
    Import {
        file: String,
    },
}

#[derive(Subcommand)]
pub enum SyncCommands {
    /// Log in (email or GitHub OAuth)
    Login,
    /// Log out
    Logout,
    /// Push config to cloud
    Push,
    /// Pull config from cloud
    Pull,
    /// Show sync status
    Status,
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        match self.command {
            Commands::Init => install::run_init().await,
            Commands::Prompt => prompt_command().await,
            Commands::Alias { action } => alias_command(action).await,
            Commands::Theme { action } => theme_command(action).await,
            Commands::Sync { action } => sync_command(action).await,
            Commands::Help { topic } => help_command(topic).await,
            Commands::Doctor => doctor_command().await,
            Commands::Bench => bench_command().await,
            Commands::Uninstall { yes } => uninstall_command(yes).await,
        }
    }
}