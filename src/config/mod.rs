use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

// ── Config root ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub prompt:   PromptConfig,
    pub colors:   ColorsConfig,
    pub icons:    IconsConfig,
    pub git:      GitConfig,
    pub alias:    AliasConfig,
    pub sync:     SyncConfig,
    pub segments: SegmentsConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            prompt:   PromptConfig::default(),
            colors:   ColorsConfig::default(),
            icons:    IconsConfig::default(),
            git:      GitConfig::default(),
            alias:    AliasConfig::default(),
            sync:     SyncConfig::default(),
            segments: SegmentsConfig::default(),
        }
    }
}

// ── Prompt ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PromptConfig {
    /// Second-line symbol  e.g. ❯ ➜ $ →
    pub symbol: String,
    /// Show prompt on two lines
    pub two_lines: bool,
    /// Show timestamp segment
    pub show_time: bool,
    /// strftime format for time
    pub time_format: String,
    /// Separator string between user@host and dir, e.g. " - "
    pub separator: String,
}

impl Default for PromptConfig {
    fn default() -> Self {
        Self {
            symbol:      "❯".to_string(),
            two_lines:   true,
            show_time:   false,
            time_format: "%H:%M".to_string(),
            separator:   " - ".to_string(),
        }
    }
}

// ── Colors — one field per visual token ──────────────────────────────────────
//
// Every field accepts:
//   "#rrggbb"  HEX (preferred)
//   "red" | "green" | "yellow" | "blue" | "magenta" | "cyan" | "white" | "gray"
//
// The renderer calls color::resolve() which handles both.

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ColorsConfig {
    // ── user segment ─────────────────────────────────────────────────────────
    pub user_icon:  String,   // color of the user icon/avatar
    pub username:   String,   // color of the username text

    // ── separator ────────────────────────────────────────────────────────────
    pub separator:  String,   // color of @ and " - " separators

    // ── hostname segment ─────────────────────────────────────────────────────
    pub hostname:   String,

    // ── directory segment ────────────────────────────────────────────────────
    pub dir_icon:   String,   // color of the directory icon
    pub directory:  String,   // color of the path text

    // ── git segment ──────────────────────────────────────────────────────────
    pub git_icon:   String,   // color of the git icon (branch icon)
    pub git_branch: String,   // color of the branch name (overridden by branch_rules)
    pub git_dirty:  String,   // color of the dirty status symbol
    pub git_staged: String,   // color of the staged status symbol
    pub git_clean:  String,   // color of the clean status symbol

    // ── prompt symbol ────────────────────────────────────────────────────────
    pub prompt_symbol: String,
}

impl Default for ColorsConfig {
    fn default() -> Self {
        Self {
            user_icon:     "#a78bfa".to_string(),
            username:      "#3fb950".to_string(),
            separator:     "#8b949e".to_string(),
            hostname:      "#58a6ff".to_string(),
            dir_icon:      "#fbbf24".to_string(),
            directory:     "#e6edf3".to_string(),
            git_icon:      "#f97316".to_string(),
            git_branch:    "#e3b341".to_string(),
            git_dirty:     "#f85149".to_string(),
            git_staged:    "#e3b341".to_string(),
            git_clean:     "#3fb950".to_string(),
            prompt_symbol: "#39d353".to_string(),
        }
    }
}

// ── Icons — glyphs/emojis for each segment ────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct IconsConfig {
    /// Icon shown before username  e.g. 🚀  (Nerd Font: \u{f007})
    pub user:       String,
    /// Icon shown before directory e.g. 📁  (Nerd Font: \u{f07b})
    pub directory:  String,
    /// Icon shown before git branch e.g. 🌿 (Nerd Font: \u{e725})
    pub git_branch: String,
    /// Status symbol when repo is dirty
    pub git_dirty:  String,
    /// Status symbol when changes are staged
    pub git_staged: String,
    /// Status symbol when repo is clean
    pub git_clean:  String,
    /// Enable Nerd Fonts glyph rendering
    pub nerd_fonts: bool,
}

impl Default for IconsConfig {
    fn default() -> Self {
        Self {
            user:       "🚀".to_string(),
            directory:  "📁".to_string(),
            git_branch: "🌿".to_string(),
            git_dirty:  "●".to_string(),
            git_staged: "●".to_string(),
            git_clean:  "●".to_string(),
            nerd_fonts: false,
        }
    }
}

// ── Git ───────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GitConfig {
    pub enabled: bool,
    /// branch pattern → color (HEX or named)
    pub branch_rules: HashMap<String, String>,
}

impl Default for GitConfig {
    fn default() -> Self {
        let mut branch_rules = HashMap::new();
        branch_rules.insert("main".to_string(),       "#f85149".to_string());
        branch_rules.insert("master".to_string(),     "#f85149".to_string());
        branch_rules.insert("develop".to_string(),    "#58a6ff".to_string());
        branch_rules.insert("feature/*".to_string(),  "#e3b341".to_string());
        branch_rules.insert("hotfix/*".to_string(),   "#bc8cff".to_string());
        branch_rules.insert("release/*".to_string(),  "#39d353".to_string());

        Self { enabled: true, branch_rules }
    }
}

// ── Alias ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct AliasConfig {
    pub enabled: bool,
}

// ── Sync ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct SyncConfig {
    pub enabled:  bool,
    pub endpoint: Option<String>,
}

// ── Segments ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SegmentsConfig {
    pub show_exit_code:    bool,
    pub show_python_venv:  bool,
    pub show_node_version: bool,
    pub shorten_dir:       bool,
    pub max_dir_depth:     usize,
}

impl Default for SegmentsConfig {
    fn default() -> Self {
        Self {
            show_exit_code:    true,
            show_python_venv:  true,
            show_node_version: false,
            shorten_dir:       true,
            max_dir_depth:     3,
        }
    }
}

// ── I/O ───────────────────────────────────────────────────────────────────────

impl Config {
    pub fn config_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("dashe")
    }

    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.toml")
    }

    pub fn exists() -> bool {
        Self::config_path().exists()
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path();
        if !path.exists() {
            return Ok(Config::default());
        }
        let raw = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config at {}", path.display()))?;
        let config: Config = toml::from_str(&raw)
            .with_context(|| "Failed to parse config.toml — run `dashe doctor` to diagnose")?;
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let dir = Self::config_dir();
        std::fs::create_dir_all(&dir)
            .with_context(|| format!("Failed to create config dir: {}", dir.display()))?;
        std::fs::write(Self::config_path(), toml::to_string_pretty(self)?)?;
        Ok(())
    }
}