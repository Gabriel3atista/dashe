use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::config::{ColorsConfig, Config};

// ── Theme definition ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub description: String,
    pub colors: ColorsConfig,
    pub prompt: PromptPartial,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PromptPartial {
    pub symbol: Option<String>,
    pub avatar: Option<String>,
    pub two_lines: Option<bool>,
}

// ── Built-in presets ──────────────────────────────────────────────────────────

pub fn builtin_themes() -> HashMap<&'static str, Theme> {
    let mut map = HashMap::new();

    map.insert(
        "p10k",
        Theme {
            name: "p10k".to_string(),
            description: "Powerlevel10k-inspired — vibrant and information-rich".to_string(),
            colors: ColorsConfig {
                username: "#af87ff".to_string(),
                hostname: "#00afff".to_string(),
                directory: "#87ff87".to_string(),
                git_branch: "#ffaf00".to_string(),
                git_dirty: "#ff5f5f".to_string(),
                git_staged: "#ffaf00".to_string(),
                git_clean: "#87ff87".to_string(),
                prompt_symbol: "#af87ff".to_string(),
                separator: "#626262".to_string(),
            },
            prompt: PromptPartial {
                symbol: Some("❯".to_string()),
                avatar: Some("⚡".to_string()),
                two_lines: Some(true),
            },
        },
    );

    map.insert(
        "starship",
        Theme {
            name: "starship".to_string(),
            description: "Starship-inspired — clean Nord palette".to_string(),
            colors: ColorsConfig {
                username: "#88c0d0".to_string(),
                hostname: "#81a1c1".to_string(),
                directory: "#5e81ac".to_string(),
                git_branch: "#ebcb8b".to_string(),
                git_dirty: "#bf616a".to_string(),
                git_staged: "#ebcb8b".to_string(),
                git_clean: "#a3be8c".to_string(),
                prompt_symbol: "#88c0d0".to_string(),
                separator: "#4c566a".to_string(),
            },
            prompt: PromptPartial {
                symbol: Some("❯".to_string()),
                avatar: Some("🌟".to_string()),
                two_lines: Some(true),
            },
        },
    );

    map.insert(
        "ocean",
        Theme {
            name: "ocean".to_string(),
            description: "Deep oceanic blues and teals".to_string(),
            colors: ColorsConfig {
                username: "#00d7ff".to_string(),
                hostname: "#0087d7".to_string(),
                directory: "#00afff".to_string(),
                git_branch: "#ffaf5f".to_string(),
                git_dirty: "#ff5f5f".to_string(),
                git_staged: "#ffaf5f".to_string(),
                git_clean: "#00d7af".to_string(),
                prompt_symbol: "#00d7ff".to_string(),
                separator: "#005f87".to_string(),
            },
            prompt: PromptPartial {
                symbol: Some("→".to_string()),
                avatar: Some("🌊".to_string()),
                two_lines: Some(true),
            },
        },
    );

    map.insert(
        "retro",
        Theme {
            name: "retro".to_string(),
            description: "Amber/green old-school terminal aesthetic".to_string(),
            colors: ColorsConfig {
                username: "#5faf00".to_string(),
                hostname: "#af8700".to_string(),
                directory: "#d7af00".to_string(),
                git_branch: "#ff8700".to_string(),
                git_dirty: "#ff5f00".to_string(),
                git_staged: "#d7af00".to_string(),
                git_clean: "#5faf00".to_string(),
                prompt_symbol: "#5faf00".to_string(),
                separator: "#585858".to_string(),
            },
            prompt: PromptPartial {
                symbol: Some("$".to_string()),
                avatar: Some("💾".to_string()),
                two_lines: Some(false),
            },
        },
    );

    map.insert(
        "minimal",
        Theme {
            name: "minimal".to_string(),
            description: "Distraction-free grayscale with subtle hints".to_string(),
            colors: ColorsConfig {
                username: "#767676".to_string(),
                hostname: "#585858".to_string(),
                directory: "#d0d0d0".to_string(),
                git_branch: "#878787".to_string(),
                git_dirty: "#af5f5f".to_string(),
                git_staged: "#af875f".to_string(),
                git_clean: "#5f875f".to_string(),
                prompt_symbol: "#9e9e9e".to_string(),
                separator: "#3a3a3a".to_string(),
            },
            prompt: PromptPartial {
                symbol: Some("▶".to_string()),
                avatar: None,
                two_lines: Some(true),
            },
        },
    );

    map
}

// ── ThemeManager ──────────────────────────────────────────────────────────────

pub struct ThemeManager {
    config: Config,
}

impl ThemeManager {
    pub fn new() -> Result<Self> {
        Ok(Self {
            config: Config::load()?,
        })
    }

    pub fn list(&self) {
        let themes = builtin_themes();
        let mut names: Vec<&&str> = themes.keys().collect();
        names.sort();

        println!("\n\x1b[96m🎨 Available Themes\x1b[0m\n");
        for name in names {
            let theme = &themes[name];
            let is_active = self.config.colors.username == theme.colors.username;
            let active_mark = if is_active { " \x1b[92m← active\x1b[0m" } else { "" };
            println!(
                "  \x1b[93m{:<12}\x1b[0m {}{active_mark}",
                name, theme.description
            );
        }
        println!("\n  \x1b[90mdashe theme set <name>\x1b[0m");
        println!();
    }

    pub fn set(&mut self, name: &str) -> Result<()> {
        let themes = builtin_themes();
        let theme = themes
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("Theme '{name}' not found. Run `dashe theme list`."))?;

        self.config.colors = theme.colors.clone();
        if let Some(sym) = &theme.prompt.symbol {
            self.config.prompt.symbol = sym.clone();
        }
        if let Some(av) = &theme.prompt.avatar {
            self.config.prompt.avatar = Some(av.clone());
        }
        if let Some(tl) = theme.prompt.two_lines {
            self.config.prompt.two_lines = tl;
        }

        self.config.save()
    }

    pub fn current(&self) {
        println!("\n\x1b[96m📋 Current Colors\x1b[0m\n");
        let c = &self.config.colors;
        let show = |label: &str, hex: &str| {
            let ansi = crate::prompt::hex_to_ansi(hex);
            println!("  {label:<18} {ansi}██████\x1b[0m  {hex}");
        };
        show("username", &c.username);
        show("hostname", &c.hostname);
        show("directory", &c.directory);
        show("git_branch", &c.git_branch);
        show("git_dirty", &c.git_dirty);
        show("git_staged", &c.git_staged);
        show("git_clean", &c.git_clean);
        show("prompt_symbol", &c.prompt_symbol);
        println!("\n  symbol: {}", self.config.prompt.symbol);
        if let Some(av) = &self.config.prompt.avatar {
            println!("  avatar: {av}");
        }
        println!();
    }

    pub fn export(&self, path: &str) -> Result<()> {
        let content = toml::to_string_pretty(&self.config)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn import(&mut self, path: &str) -> Result<()> {
        let raw = std::fs::read_to_string(path)
            .with_context(|| format!("Cannot read: {path}"))?;
        let imported: Config =
            toml::from_str(&raw).with_context(|| "Invalid theme file format")?;
        self.config.colors = imported.colors;
        self.config.prompt.symbol = imported.prompt.symbol;
        self.config.prompt.avatar = imported.prompt.avatar;
        self.config.save()
    }
}