use anyhow::Result;
use console::style;
use dialoguer::{Confirm, Input, Select};
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

use crate::config::Config;

const DASHE_ASCII: &str = r#"
██████╗  █████╗ ███████╗██╗  ██╗███████╗
██╔══██╗██╔══██╗██╔════╝██║  ██║██╔════╝
██║  ██║███████║███████╗███████║█████╗  
██║  ██║██╔══██║╚════██║██╔══██║██╔══╝  
██████╔╝██║  ██║███████║██║  ██║███████╗
╚═════╝ ╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝╚══════╝
"#;

pub async fn run_init() -> Result<()> {
    // Print branded header
    println!("\x1b[96m{}\x1b[0m", DASHE_ASCII);
    println!("{}", style("⚡ Modern Terminal Customizer for Linux/Bash").cyan().bold());
    println!("{}", style("   Version 0.1.0\n").dim());

    // Check if already configured
    if Config::exists() {
        let overwrite = Confirm::new()
            .with_prompt("Dashe is already configured. Overwrite?")
            .default(false)
            .interact()?;

        if !overwrite {
            println!("\n{}", style("Keeping existing config. Run `dashe theme set <name>` to change themes.").dim());
            return Ok(());
        }
    }

    println!("{}\n", style("Let's set up your perfect terminal! 🚀").bold());

    // Step 1: Pick avatar/emoji
    let avatar: String = Input::new()
        .with_prompt("Your avatar emoji (press Enter for default)")
        .default("🚀".to_string())
        .interact_text()?;

    // Step 2: Pick a preset theme
    let themes = vec!["default"];
    let theme_idx = Select::new()
        .with_prompt("Choose a theme preset")
        .items(&themes)
        .default(0)
        .interact()?;
    let chosen_theme = themes[theme_idx];

    // Step 3: Prompt symbol
    let symbols = vec!["❯", "➜", "→", "$", "#", "▶"];
    let sym_idx = Select::new()
        .with_prompt("Choose your prompt symbol")
        .items(&symbols)
        .default(0)
        .interact()?;
    let prompt_symbol = symbols[sym_idx];

    // Step 4: Enable Git integration?
    let git_enabled = Confirm::new()
        .with_prompt("Enable Git integration?")
        .default(true)
        .interact()?;

    // Step 5: Enable cloud sync?
    let sync_enabled = Confirm::new()
        .with_prompt("Enable cloud sync? (optional, requires account)")
        .default(false)
        .interact()?;

    println!();

    // Animate installation
    let pb = ProgressBar::new(100);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.cyan} [{bar:40.cyan/blue}] {msg}",
        )?
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
        .progress_chars("█▉▊▋▌▍▎▏  "),
    );

    let steps = vec![
        (20, "Creating config directory..."),
        (40, "Writing configuration..."),
        (60, "Applying theme..."),
        (75, "Generating shell integration files..."),
        (90, "Setting up alias system..."),
        (100, "Done!"),
    ];

    for (progress, msg) in steps {
        pb.set_message(msg);
        pb.set_position(progress);
        tokio::time::sleep(Duration::from_millis(120)).await;
    }
    pb.finish_with_message("Installation complete! ✨");

    // Build and save config
    let mut config = Config::default();
    config.icons.user = avatar;
    config.prompt.symbol = prompt_symbol.to_string();
    config.git.enabled = git_enabled;
    config.sync.enabled = sync_enabled;

    // Apply theme preset colors
    apply_theme_to_config(&mut config, chosen_theme);
    config.save()?;

    // Generate alias shell file
    let alias_mgr = crate::alias::AliasManager::load()?;
    alias_mgr.write_shell_file()?;

    println!("\n{}", style("✅ Dashe is ready!").green().bold());
    println!("\n{}", style("Add this to your ~/.bashrc:").bold());
    println!("\x1b[96m  export PROMPT_COMMAND='PS1=\"$(dashe prompt)\"'\x1b[0m");
    println!("\x1b[96m  source ~/.config/dashe/aliases.sh\x1b[0m");
    println!("\nThen run: {}", style("source ~/.bashrc").cyan());
    println!("\n{}  dashe help", style("📖 Help:").dim());
    println!("{}  dashe alias add gs \"git status\"", style("💡 Try:").dim());
    println!("{}  dashe theme list", style("🎨 Themes:").dim());

    Ok(())
}

fn apply_theme_to_config(config: &mut Config, theme: &str) {
    use crate::theme::builtin_themes;
    let themes = builtin_themes();
    if let Some(t) = themes.into_iter().find(|t| t.name == theme) {
        t.apply_to(config);
    }
}