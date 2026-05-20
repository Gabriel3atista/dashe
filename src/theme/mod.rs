//! Theme manager.
//!
//! A theme is a complete snapshot of [colors] + [icons] + relevant [prompt] fields.
//! Import applies ALL fields — nothing is silently ignored.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::config::{
    AliasConfig,
    ColorsConfig,
    Config,
    GitConfig,
    IconsConfig,
    PromptConfig,
    SegmentsConfig,
    SyncConfig,
};

// ── Theme file schema ─────────────────────────────────────────────────────────
//
// This is what gets serialized to / deserialized from a theme TOML file.
// Only the fields a theme cares about — not the full Config.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub description: String,
    pub prompt: PromptConfig,
    pub colors: ColorsConfig,
    pub icons: IconsConfig,
    pub git: GitConfig,
    pub alias: AliasConfig,
    pub sync: SyncConfig,
    pub segments: SegmentsConfig,
}

// ── Apply a theme to a Config ─────────────────────────────────────────────────

impl Theme {
    pub fn apply_to(&self, cfg: &mut Config) {
        cfg.prompt = self.prompt.clone();
        cfg.colors = self.colors.clone();
        cfg.icons = self.icons.clone();
        cfg.git = self.git.clone();
        cfg.alias = self.alias.clone();
        cfg.sync = self.sync.clone();
        cfg.segments = self.segments.clone();
    }
}

// ── Built-in presets ──────────────────────────────────────────────────────────

pub fn builtin_themes() -> Vec<Theme> {
    vec![
        Theme {
            name:        "default".to_string(),
            description: "Dashe theme — vibrant, information-rich".to_string(),
            colors: ColorsConfig {
                user_icon:     "#af87ff".to_string(),
                username:      "#af87ff".to_string(),
                separator:     "#626262".to_string(),
                hostname:      "#00afff".to_string(),
                dir_icon:      "#ffaf00".to_string(),
                directory:     "#87ff87".to_string(),
                git_icon:      "#ff8700".to_string(),
                git_branch:    "#ffaf00".to_string(),
                git_dirty:     "#ff5f5f".to_string(),
                git_staged:    "#ffaf00".to_string(),
                git_clean:     "#87ff87".to_string(),
                prompt_symbol: "#af87ff".to_string(),
            },
            icons: IconsConfig {
                user:       "⚡".to_string(),
                directory:  "X".to_string(),   // Nerd Font folder
                git_branch: "".to_string(),   // Nerd Font branch
                git_dirty:  "✗".to_string(),
                git_staged: "✚".to_string(),
                git_clean:  "✔".to_string(),
                nerd_fonts: false,
            },
            prompt: PromptConfig {
                symbol:      "❯".to_string(),
                two_lines:   true,
                show_time:   false,
                time_format: "%H:%M".to_string(),
                separator:   " - ".to_string(),
            },
            git: GitConfig::default(),
            alias: AliasConfig::default(),
            sync: SyncConfig::default(),
            segments: SegmentsConfig::default(),
        },
    ]
}

// ── ThemeManager ──────────────────────────────────────────────────────────────

pub struct ThemeManager {
    config: Config,
}

impl ThemeManager {
    pub fn new() -> Result<Self> {
        Ok(Self { config: Config::load()? })
    }

    pub fn list(&self) {
        println!("\n\x1b[96m🎨 Available Themes\x1b[0m\n");
        for t in builtin_themes() {
            // Detect active by matching username color (stable identifier)
            let active = self.config.colors.username == t.colors.username;
            let mark   = if active { " \x1b[92m← active\x1b[0m" } else { "" };
            println!("  \x1b[93m{:<12}\x1b[0m {}{mark}", t.name, t.description);
        }
        println!("\n  \x1b[90mdashe theme set <name>\x1b[0m\n");
    }

    pub fn set(&mut self, name: &str) -> Result<()> {
        let theme = builtin_themes()
            .into_iter()
            .find(|t| t.name == name)
            .ok_or_else(|| anyhow::anyhow!("Theme '{name}' not found. Run `dashe theme list`."))?;

        theme.apply_to(&mut self.config);
        self.config.save()
    }

    /// Show each color token with an inline preview swatch
    pub fn current(&self) {
        let c = &self.config.colors;
        let i = &self.config.icons;

        println!("\n\x1b[96m📋 Current Theme\x1b[0m\n");
        println!("  \x1b[90m── Colors ───────────────────────────────────────\x1b[0m");

        let show = |label: &str, hex: &str| {
            let ansi = crate::prompt::resolve(hex);
            println!("  {label:<18} {ansi}██████\x1b[0m  {hex}");
        };

        show("user_icon",     &c.user_icon);
        show("username",      &c.username);
        show("separator",     &c.separator);
        show("hostname",      &c.hostname);
        show("dir_icon",      &c.dir_icon);
        show("directory",     &c.directory);
        show("git_icon",      &c.git_icon);
        show("git_branch",    &c.git_branch);
        show("git_dirty",     &c.git_dirty);
        show("git_staged",    &c.git_staged);
        show("git_clean",     &c.git_clean);
        show("prompt_symbol", &c.prompt_symbol);

        println!("\n  \x1b[90m── Icons ────────────────────────────────────────\x1b[0m");
        println!("  {:<18} {}", "user",       i.user);
        println!("  {:<18} {}", "directory",  i.directory);
        println!("  {:<18} {}", "git_branch", i.git_branch);
        println!("  {:<18} {}", "git_dirty",  i.git_dirty);
        println!("  {:<18} {}", "git_staged", i.git_staged);
        println!("  {:<18} {}", "git_clean",  i.git_clean);

        println!("\n  \x1b[90m── Prompt ───────────────────────────────────────\x1b[0m");
        println!("  {:<18} {}", "symbol",     self.config.prompt.symbol);
        println!("  {:<18} {}", "two_lines",  self.config.prompt.two_lines);
        println!();
    }

    /// Export just the theme-relevant fields (not the full config) to a file
    pub fn export(&self, path: &str) -> Result<()> {
        let theme = Theme {
            name: "custom".to_string(),
            description: "Exported from Dashe".to_string(),
            prompt: self.config.prompt.clone(),
            colors: self.config.colors.clone(),
            icons: self.config.icons.clone(),
            git: self.config.git.clone(),
            alias: self.config.alias.clone(),
            sync: self.config.sync.clone(),
            segments: self.config.segments.clone(),
        };

        std::fs::write(path, toml::to_string_pretty(&theme)?)?;
        Ok(())
    }

    /// Import a theme file — applies ALL fields, saves full config
    pub fn import(&mut self, path: &str) -> Result<()> {
        let raw    = std::fs::read_to_string(path)
            .with_context(|| format!("Cannot read: {path}"))?;
        let theme: Theme = toml::from_str(&raw)
            .with_context(|| "Invalid theme file. Expected [colors], [icons], [prompt] sections.")?;

        theme.apply_to(&mut self.config);
        self.config.save()
    }
}