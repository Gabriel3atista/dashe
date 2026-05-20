use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

// ── Config root ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub prompt: PromptConfig,
    pub colors: ColorsConfig,
    pub git: GitConfig,
    pub alias: AliasConfig,
    pub sync: SyncConfig,
    pub icons: IconConfig,
    pub segments: SegmentsConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            prompt: PromptConfig::default(),
            colors: ColorsConfig::default(),
            git: GitConfig::default(),
            alias: AliasConfig::default(),
            sync: SyncConfig::default(),
            icons: IconConfig::default(),
            segments: SegmentsConfig::default(),
        }
    }
}

// ── Prompt ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PromptConfig {
    /// The prompt symbol (e.g. ❯, ➜, $)
    pub symbol: String,
    /// User avatar/emoji prefix
    pub avatar: Option<String>,
    /// Ordered list of segments to display
    pub layout: Vec<String>,
    /// Show prompt on two lines (segment line + symbol line)
    pub two_lines: bool,
    /// Show timestamp in prompt
    pub show_time: bool,
    /// Time format string (strftime)
    pub time_format: String,
}

impl Default for PromptConfig {
    fn default() -> Self {
        Self {
            symbol: "❯".to_string(),
            avatar: Some("🚀".to_string()),
            layout: vec![
                "avatar".to_string(),
                "user".to_string(),
                "hostname".to_string(),
                "dir".to_string(),
                "git_branch".to_string(),
                "git_status".to_string(),
            ],
            two_lines: true,
            show_time: false,
            time_format: "%H:%M:%S".to_string(),
        }
    }
}

// ── Colors ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ColorsConfig {
    pub username: String,
    pub hostname: String,
    pub directory: String,
    pub git_branch: String,
    pub git_dirty: String,
    pub git_staged: String,
    pub git_clean: String,
    pub prompt_symbol: String,
    pub separator: String,
}

impl Default for ColorsConfig {
    fn default() -> Self {
        Self {
            username: "#3fb950".to_string(),
            hostname: "#58a6ff".to_string(),
            directory: "#e6edf3".to_string(),
            git_branch: "#e3b341".to_string(),
            git_dirty: "#f85149".to_string(),
            git_staged: "#e3b341".to_string(),
            git_clean: "#3fb950".to_string(),
            prompt_symbol: "#39d353".to_string(),
            separator: "#8b949e".to_string(),
        }
    }
}

// ── Git ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GitConfig {
    pub enabled: bool,
    /// Map branch pattern → color name/hex
    pub branch_rules: HashMap<String, String>,
    pub status: GitStatusConfig,
}

impl Default for GitConfig {
    fn default() -> Self {
        let mut branch_rules = HashMap::new();
        branch_rules.insert("main".to_string(), "red".to_string());
        branch_rules.insert("master".to_string(), "red".to_string());
        branch_rules.insert("develop".to_string(), "blue".to_string());
        branch_rules.insert("feature/*".to_string(), "yellow".to_string());
        branch_rules.insert("hotfix/*".to_string(), "magenta".to_string());
        branch_rules.insert("release/*".to_string(), "cyan".to_string());

        Self {
            enabled: true,
            branch_rules,
            status: GitStatusConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GitStatusConfig {
    pub dirty: String,
    pub staged: String,
    pub clean: String,
    pub show_count: bool,
}

impl Default for GitStatusConfig {
    fn default() -> Self {
        Self {
            dirty: "●".to_string(),
            staged: "●".to_string(),
            clean: "●".to_string(),
            show_count: false,
        }
    }
}

// ── Alias ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct AliasConfig {
    pub enabled: bool,
}

// ── Sync ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct SyncConfig {
    pub enabled: bool,
    pub endpoint: Option<String>,
}

// ── Icons ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct IconConfig {
    pub git_branch: String,
    pub git_dirty: String,
    pub git_staged: String,
    pub git_clean: String,
    pub nerd_fonts: bool,
}

impl Default for IconConfig {
    fn default() -> Self {
        Self {
            git_branch: "".to_string(), // Nerd Font branch icon
            git_dirty: "✗".to_string(),
            git_staged: "✚".to_string(),
            git_clean: "✔".to_string(),
            nerd_fonts: false,
        }
    }
}

// ── Segments ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SegmentsConfig {
    pub show_exit_code: bool,
    pub show_python_venv: bool,
    pub show_node_version: bool,
    pub shorten_dir: bool,
    pub max_dir_depth: usize,
}

impl Default for SegmentsConfig {
    fn default() -> Self {
        Self {
            show_exit_code: true,
            show_python_venv: true,
            show_node_version: false,
            shorten_dir: true,
            max_dir_depth: 3,
        }
    }
}

// ── I/O ──────────────────────────────────────────────────────────────────────

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

        let content = toml::to_string_pretty(self)?;
        std::fs::write(Self::config_path(), content)?;
        Ok(())
    }
}
