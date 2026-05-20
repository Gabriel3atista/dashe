use anyhow::Result;
use std::time::Instant;

use crate::alias::AliasManager;
use crate::config::Config;
use crate::doctor;
use crate::prompt::Renderer;
use crate::sync::SyncClient;
use crate::theme::ThemeManager;

use super::{AliasCommands, SyncCommands, ThemeCommands};

// ── prompt ──────────────────────────────────────────────────────────────────

pub async fn prompt_command() -> Result<()> {
    let config = Config::load()?;
    let renderer = Renderer::new(config);
    let output = renderer.render().await?;
    // Print raw — this is eval'd by bash via PROMPT_COMMAND
    print!("{output}");
    Ok(())
}

// ── alias ────────────────────────────────────────────────────────────────────

pub async fn alias_command(action: AliasCommands) -> Result<()> {
    let mut manager = AliasManager::load()?;

    match action {
        AliasCommands::Add { key, command, category } => {
            manager.add(&key, &command, category.as_deref())?;
            println!("✅ Alias \x1b[92m{key}\x1b[0m → \x1b[96m{command}\x1b[0m added.");
            manager.write_shell_file()?;
        }
        AliasCommands::Remove { key } => {
            manager.remove(&key)?;
            println!("🗑  Alias \x1b[91m{key}\x1b[0m removed.");
            manager.write_shell_file()?;
        }
        AliasCommands::List { category } => {
            manager.list(category.as_deref());
        }
        AliasCommands::Export { output } => {
            manager.export(&output)?;
            println!("📦 Aliases exported to \x1b[96m{output}\x1b[0m");
        }
        AliasCommands::Import { file } => {
            let count = manager.import(&file)?;
            println!("📥 Imported \x1b[92m{count}\x1b[0m aliases from \x1b[96m{file}\x1b[0m");
            manager.write_shell_file()?;
        }
    }
    Ok(())
}

// ── theme ────────────────────────────────────────────────────────────────────

pub async fn theme_command(action: ThemeCommands) -> Result<()> {
    let mut tm = ThemeManager::new()?;

    match action {
        ThemeCommands::List => tm.list(),
        ThemeCommands::Set { name } => {
            tm.set(&name)?;
            println!("🎨 Theme \x1b[92m{name}\x1b[0m applied. Restart your shell or run:");
            println!("   \x1b[96meval \"$(dashe prompt)\"\x1b[0m");
        }
        ThemeCommands::Current => tm.current(),
        ThemeCommands::Export { output } => {
            tm.export(&output)?;
            println!("📦 Theme exported to \x1b[96m{output}\x1b[0m");
        }
        ThemeCommands::Import { file } => {
            tm.import(&file)?;
            println!("📥 Theme imported from \x1b[96m{file}\x1b[0m");
        }
    }
    Ok(())
}

// ── sync ─────────────────────────────────────────────────────────────────────

pub async fn sync_command(action: SyncCommands) -> Result<()> {
    let client = SyncClient::new()?;

    match action {
        SyncCommands::Login => client.login().await?,
        SyncCommands::Logout => client.logout()?,
        SyncCommands::Push => client.push().await?,
        SyncCommands::Pull => client.pull().await?,
        SyncCommands::Status => client.status()?,
    }
    Ok(())
}

// ── help ─────────────────────────────────────────────────────────────────────

pub async fn help_command(topic: Option<String>) -> Result<()> {
    match topic.as_deref() {
        None => print_main_help(),
        Some("alias") => print_alias_help(),
        Some("theme") => print_theme_help(),
        Some("git") => print_git_help(),
        Some("sync") => print_sync_help(),
        Some("prompt") => print_prompt_help(),
        Some(other) => {
            println!("\x1b[91mUnknown help topic: {other}\x1b[0m");
            println!("Available topics: alias, theme, git, sync, prompt");
        }
    }
    Ok(())
}

fn print_main_help() {
    println!(
        r#"
\x1b[96m⚡ Dashe — Terminal Customizer\x1b[0m

\x1b[93mCOMMANDS\x1b[0m
  dashe init              Interactive setup wizard
  dashe prompt            Render current prompt (use in PROMPT_COMMAND)
  dashe alias <action>    Manage shell aliases
  dashe theme <action>    Manage visual themes
  dashe sync <action>     Cloud sync & backup
  dashe doctor            Diagnose environment issues
  dashe bench             Benchmark startup performance

\x1b[93mQUICK START\x1b[0m
  1. dashe init
  2. Add to ~/.bashrc:
     export PROMPT_COMMAND='PS1="$(dashe prompt)"'

\x1b[93mHELP TOPICS\x1b[0m
  dashe help alias | theme | git | sync | prompt
"#
    );
}

fn print_alias_help() {
    println!(
        r#"
\x1b[96m📌 Alias Management\x1b[0m

\x1b[93mEXAMPLES\x1b[0m
  dashe alias add gs "git status"
  dashe alias add c "clear" --category shell
  dashe alias list
  dashe alias list --category git
  dashe alias remove gs
  dashe alias export my-aliases.toml
  dashe alias import my-aliases.toml

\x1b[93mNOTES\x1b[0m
  Aliases are saved to ~/.config/dashe/aliases.toml
  A shell file is auto-generated at ~/.config/dashe/aliases.sh
  Source it in your ~/.bashrc:
    source ~/.config/dashe/aliases.sh
"#
    );
}

fn print_theme_help() {
    println!(
        r#"
\x1b[96m🎨 Theme Management\x1b[0m

\x1b[93mPRESETS\x1b[0m
  p10k      Powerlevel10k-inspired
  starship  Starship-inspired minimal
  ocean     Blue/teal oceanic
  retro     Amber/green retro terminal
  minimal   Clean and distraction-free

\x1b[93mEXAMPLES\x1b[0m
  dashe theme list
  dashe theme set ocean
  dashe theme export my-theme.toml
  dashe theme import my-theme.toml
"#
    );
}

fn print_git_help() {
    println!(
        r#"
\x1b[96m🌿 Git Integration\x1b[0m

\x1b[93mBRANCH COLORS\x1b[0m
  main/master  → Red
  Any other    → Yellow (configurable)

\x1b[93mSTATUS INDICATORS\x1b[0m
  🔴 ●  Dirty (uncommitted changes)
  🟡 ●  Staged (ready to commit)
  🟢 ●  Clean

\x1b[93mCONFIG EXAMPLE\x1b[0m
  [git.branch_rules]
  main       = "red"
  develop    = "blue"
  "feature/*" = "yellow"
  "hotfix/*"  = "magenta"

  [git.status]
  dirty  = "🔴"
  staged = "🟡"
  clean  = "🟢"
"#
    );
}

fn print_sync_help() {
    println!(
        r#"
\x1b[96m☁️  Cloud Sync\x1b[0m

\x1b[93mCOMMANDS\x1b[0m
  dashe sync login    Authenticate (email or GitHub OAuth)
  dashe sync push     Upload current config to cloud
  dashe sync pull     Download config from cloud
  dashe sync logout   Clear credentials
  dashe sync status   Show sync state

\x1b[93mNOTES\x1b[0m
  Credentials are stored securely in the system keyring.
  Dashe works fully offline — sync is opt-in.
"#
    );
}

fn print_prompt_help() {
    println!(
        r##"
\x1b[96m💻 Prompt Customization\x1b[0m

\x1b[93mSETUP\x1b[0m
  Add to ~/.bashrc:
    export PROMPT_COMMAND='PS1="$(dashe prompt)"'

\x1b[93mCONFIG EXAMPLE\x1b[0m
  [prompt]
  symbol  = "❯"
  avatar  = "🚀"
  layout  = "avatar user hostname dir git_branch git_status"

  [colors]
  username  = "#3fb950"
  hostname  = "#58a6ff"
  directory = "#e6edf3"

\x1b[93mSEGMENTS\x1b[0m
  avatar, user, hostname, dir, git_branch, git_status,
  time, python_venv, node_version, exit_code
"##
    );
}

// ── doctor ───────────────────────────────────────────────────────────────────

pub async fn doctor_command() -> Result<()> {
    doctor::run_checks()
}

// ── bench ────────────────────────────────────────────────────────────────────

pub async fn bench_command() -> Result<()> {
    println!("\x1b[96m⚡ Dashe Startup Benchmark\x1b[0m\n");

    let iterations = 100u32;
    let mut times = Vec::with_capacity(iterations as usize);

    for _ in 0..iterations {
        let start = Instant::now();
        let config = Config::load()?;
        let renderer = Renderer::new(config);
        let _ = renderer.render().await?;
        times.push(start.elapsed().as_micros());
    }

    times.sort_unstable();
    let median = times[iterations as usize / 2];
    let p95 = times[(iterations as usize * 95) / 100];
    let min = times[0];
    let max = times[iterations as usize - 1];

    println!("  Iterations : {iterations}");
    println!("  Min        : {min}µs  ({:.2}ms)", min as f64 / 1000.0);
    println!("  Median     : {median}µs  ({:.2}ms)", median as f64 / 1000.0);
    println!("  p95        : {p95}µs  ({:.2}ms)", p95 as f64 / 1000.0);
    println!("  Max        : {max}µs  ({:.2}ms)", max as f64 / 1000.0);

    if median < 5000 {
        println!("\n  \x1b[92m✅ Excellent! Under 5ms median.\x1b[0m");
    } else if median < 20000 {
        println!("\n  \x1b[93m⚠️  Acceptable, but could be optimized.\x1b[0m");
    } else {
        println!("\n  \x1b[91m❌ Slow! Run `dashe doctor` to investigate.\x1b[0m");
    }

    Ok(())
}

// ── uninstall ────────────────────────────────────────────────────────────────

pub async fn uninstall_command(yes: bool) -> Result<()> {
    println!("\x1b[96m🗑  Dashe Uninstaller\x1b[0m\n");

    let config_dir = Config::config_dir();
    let binary_paths = vec![
        std::path::PathBuf::from("/usr/local/bin/dashe"),
        std::path::PathBuf::from("/usr/bin/dashe"),
    ];
    let binary_path = binary_paths.iter().find(|p| p.exists());

    // Show what will be removed
    println!("  The following will be removed:\n");

    if let Some(bin) = &binary_path {
        println!("  \x1b[91m✗\x1b[0m  Binary      {}", bin.display());
    } else {
        println!("  \x1b[90m–\x1b[0m  Binary      not found in /usr/local/bin or /usr/bin");
    }

    if config_dir.exists() {
        println!("  \x1b[91m✗\x1b[0m  Config dir  {}", config_dir.display());
        // List contents
        if let Ok(entries) = std::fs::read_dir(&config_dir) {
            for entry in entries.flatten() {
                println!("              └── {}", entry.file_name().to_string_lossy());
            }
        }
    } else {
        println!("  \x1b[90m–\x1b[0m  Config dir  not found");
    }

    println!();
    println!("  \x1b[93m⚠️  Remember to remove these lines from ~/.bashrc manually:\x1b[0m");
    println!("  \x1b[90mexport PROMPT_COMMAND=\"DASHE_EXIT=\\$?; PS1=\\\"\\$(dashe prompt)\\\"\"\x1b[0m");
    println!("  \x1b[90msource ~/.config/dashe/aliases.sh\x1b[0m");
    println!();

    // Confirm
    let confirmed = if yes {
        true
    } else {
        dialoguer::Confirm::new()
            .with_prompt("Proceed with uninstall?")
            .default(false)
            .interact()?
    };

    if !confirmed {
        println!("\n  Uninstall cancelled.");
        return Ok(());
    }

    println!();

    // Remove binary (needs sudo — try, inform if fails)
    if let Some(bin) = binary_path {
        match std::fs::remove_file(bin) {
            Ok(_) => println!("  \x1b[92m✓\x1b[0m  Removed binary {}", bin.display()),
            Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                println!("  \x1b[93m⚠\x1b[0m  Permission denied removing binary — run:");
                println!("      \x1b[96msudo rm {}\x1b[0m", bin.display());
            }
            Err(e) => println!("  \x1b[91m✗\x1b[0m  Could not remove binary: {e}"),
        }
    }

    // Remove config directory
    if config_dir.exists() {
        match std::fs::remove_dir_all(&config_dir) {
            Ok(_) => println!("  \x1b[92m✓\x1b[0m  Removed config dir {}", config_dir.display()),
            Err(e) => println!("  \x1b[91m✗\x1b[0m  Could not remove config dir: {e}"),
        }
    }

    // Auto-clean ~/.bashrc
    let bashrc = dirs::home_dir()
        .unwrap_or_default()
        .join(".bashrc");

    if bashrc.exists() {
        let content = std::fs::read_to_string(&bashrc)?;
        let cleaned: String = content
            .lines()
            .filter(|line| !line.contains("dashe"))
            .map(|line| format!("{line}\n"))
            .collect();

        if cleaned != content {
            std::fs::write(&bashrc, &cleaned)?;
            println!("  \x1b[92m✓\x1b[0m  Cleaned dashe entries from ~/.bashrc");
        }
    }

    println!("\n  \x1b[92m✅ Dashe uninstalled successfully.\x1b[0m");
    println!("  Run \x1b[96msource ~/.bashrc\x1b[0m to restore your default prompt.\n");

    Ok(())
}