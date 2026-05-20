//! Prompt renderer.
//!
//! Fixed PS1 structure:
//!   [user_icon] [username][separator][hostname] - [dir_icon][dir] - ([git_icon][branch][status])
//!   [symbol]
//!
//! Every color token is read from ColorsConfig (hex preferred, named fallback).

use anyhow::Result;
use std::env;
use std::path::PathBuf;

use crate::config::Config;
use crate::git;

// ── Renderer ──────────────────────────────────────────────────────────────────

pub struct Renderer {
    config: Config,
}

impl Renderer {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub async fn render(&self) -> Result<String> {
        let cfg  = &self.config;
        let col  = &cfg.colors;
        let ico  = &cfg.icons;
        let r    = "\x1b[0m"; // reset

        // ── raw data ──────────────────────────────────────────────────────────
        let username = env::var("USER").unwrap_or_else(|_| "user".to_string());
        let hostname = read_hostname();
        let dir      = current_dir_display(cfg.segments.max_dir_depth, cfg.segments.shorten_dir);
        let git      = if cfg.git.enabled { git::detect() } else { git::GitInfo::default() };

        // ── color escapes ─────────────────────────────────────────────────────
        let c_user_icon  = resolve(&col.user_icon);
        let c_username   = resolve(&col.username);
        let c_sep        = resolve(&col.separator);
        let c_hostname   = resolve(&col.hostname);
        let c_dir_icon   = resolve(&col.dir_icon);
        let c_dir        = resolve(&col.directory);
        let c_sym        = resolve(&col.prompt_symbol);

        // ── segment: [user_icon] username@hostname ────────────────────────────
        let user_seg = format!(
            "{c_user_icon}{icon}{r} {c_username}{username}{r}{c_sep}@{r}{c_hostname}{hostname}{r}",
            icon = ico.user,
        );

        // ── segment: [dir_icon] dir ───────────────────────────────────────────
        let dir_seg = format!(
            "{c_dir_icon}{icon}{r}{c_dir}{dir}{r}",
            icon = ico.directory,
        );

        // ── segment: (git_icon branch status) — only inside a repo ───────────
        let git_seg = if git.in_repo {
            if let Some(branch) = &git.branch {
                let c_git_icon   = resolve(&col.git_icon);
                let c_branch     = git_branch_color(branch, cfg);
                let status_part  = git_status_str(&git, cfg);
                let upstream     = git_upstream_str(&git);

                format!(
                    "({c_git_icon}{icon}{r}{c_branch}{branch}{r}{status_part}{upstream})",
                    icon = ico.git_branch,
                )
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        // ── optional extra segments ───────────────────────────────────────────
        let extras = build_extras(cfg, r);

        // ── assemble line 1 ───────────────────────────────────────────────────
        let sep = format!("{c_sep} - {r}");

        let mut parts = vec![user_seg, dir_seg];
        if !git_seg.is_empty() {
            parts.push(git_seg);
        }
        if !extras.is_empty() {
            parts.push(extras);
        }
        let line1 = parts.join(&sep);

        // ── line 2 (symbol) or inline ─────────────────────────────────────────
        let output = if cfg.prompt.two_lines {
            format!("{line1}\n{c_sym}{sym}{r} ", sym = cfg.prompt.symbol)
        } else {
            format!("{line1} {c_sym}{sym}{r} ", sym = cfg.prompt.symbol)
        };

        Ok(output)
    }
}

// ── Git helpers ───────────────────────────────────────────────────────────────

/// Resolve branch color: branch_rules (HEX or named) → ColorsConfig.git_branch fallback
fn git_branch_color(branch: &str, cfg: &Config) -> String {
    use glob::Pattern;

    for (pattern, color) in &cfg.git.branch_rules {
        let matches = pattern == branch
            || Pattern::new(pattern).map(|p| p.matches(branch)).unwrap_or(false);
        if matches {
            return resolve(color);
        }
    }
    resolve(&cfg.colors.git_branch)
}

fn git_status_str(git: &git::GitInfo, cfg: &Config) -> String {
    let r    = "\x1b[0m";
    let col  = &cfg.colors;
    let ico  = &cfg.icons;

    match git.status {
        git::RepoStatus::Dirty  => format!(" {}{}{r}", resolve(&col.git_dirty),  ico.git_dirty),
        git::RepoStatus::Staged => format!(" {}{}{r}", resolve(&col.git_staged), ico.git_staged),
        git::RepoStatus::Clean  => format!(" {}{}{r}", resolve(&col.git_clean),  ico.git_clean),
    }
}

fn git_upstream_str(git: &git::GitInfo) -> String {
    let r = "\x1b[0m";
    let mut parts = Vec::new();
    if git.ahead  > 0 { parts.push(format!("\x1b[92m↑{}{r}", git.ahead));  }
    if git.behind > 0 { parts.push(format!("\x1b[91m↓{}{r}", git.behind)); }
    if parts.is_empty() { String::new() } else { format!(" {}", parts.join(" ")) }
}

// ── Optional extras ───────────────────────────────────────────────────────────

fn build_extras(cfg: &Config, r: &str) -> String {
    let mut parts = Vec::new();

    if cfg.segments.show_python_venv {
        if let Ok(venv) = env::var("VIRTUAL_ENV") {
            let name = PathBuf::from(venv)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            parts.push(format!("\x1b[33m🐍 {name}{r}"));
        }
    }

    if cfg.segments.show_node_version {
        if let Some(v) = node_version() {
            parts.push(format!("\x1b[32m⬡ {v}{r}"));
        }
    }

    if cfg.prompt.show_time {
        let now = chrono::Local::now();
        let t   = now.format(&cfg.prompt.time_format).to_string();
        let c   = resolve(&cfg.colors.separator);
        parts.push(format!("{c}🕐 {t}{r}"));
    }

    if cfg.segments.show_exit_code {
        if let Ok(code) = env::var("DASHE_EXIT") {
            if code != "0" && !code.is_empty() {
                parts.push(format!("\x1b[91m✘ {code}{r}"));
            }
        }
    }

    parts.join(" ")
}

// ── System helpers ────────────────────────────────────────────────────────────

fn read_hostname() -> String {
    std::fs::read_to_string("/etc/hostname")
        .map(|s| s.trim().to_string())
        .or_else(|_| env::var("HOSTNAME"))
        .unwrap_or_else(|_| "localhost".to_string())
}

fn current_dir_display(max_depth: usize, shorten: bool) -> String {
    let cwd  = env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/home"));

    let display = if let Ok(rel) = cwd.strip_prefix(&home) {
        if rel.as_os_str().is_empty() { "~".to_string() }
        else { format!("~/{}", rel.display()) }
    } else {
        cwd.display().to_string()
    };

    if shorten && max_depth > 0 { shorten_path(&display, max_depth) }
    else { display }
}

fn shorten_path(path: &str, max_depth: usize) -> String {
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() <= max_depth + 1 { return path.to_string(); }
    let visible = &parts[parts.len() - max_depth..];
    let prefix  = if path.starts_with('~') { "~" } else { "" };
    format!("{prefix}/…/{}", visible.join("/"))
}

fn node_version() -> Option<String> {
    if let Ok(v) = std::fs::read_to_string(".nvmrc") {
        return Some(v.trim().to_string());
    }
    std::process::Command::new("node")
        .arg("--version")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().trim_start_matches('v').to_string())
}

// ── Color resolution — the single source of truth ────────────────────────────

/// Resolve any color string to an ANSI escape sequence.
///
/// Accepts:
///   "#rrggbb"  → true-color RGB
///   "#rgb"     → short hex (expanded)
///   "red" | "green" | "yellow" | "blue" | "magenta" | "cyan" | "white" | "gray"
///   ""         → reset (empty string treated as "no color")
pub fn resolve(color: &str) -> String {
    let color = color.trim();
    if color.is_empty() {
        return "\x1b[0m".to_string();
    }

    if let Some(hex) = color.strip_prefix('#') {
        // Short hex #rgb → expand to #rrggbb
        let expanded: String = if hex.len() == 3 {
            hex.chars().flat_map(|c| [c, c]).collect()
        } else {
            hex.to_string()
        };

        if expanded.len() == 6 {
            if let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&expanded[0..2], 16),
                u8::from_str_radix(&expanded[2..4], 16),
                u8::from_str_radix(&expanded[4..6], 16),
            ) {
                return format!("\x1b[38;2;{r};{g};{b}m");
            }
        }
        // Malformed hex — fall through to gray fallback
        tracing::warn!("Invalid hex color '{color}', falling back to gray");
        return "\x1b[90m".to_string();
    }

    // Named colors
    match color.to_lowercase().as_str() {
        "red"            => "\x1b[31m",
        "bright_red"     => "\x1b[91m",
        "green"          => "\x1b[32m",
        "bright_green"   => "\x1b[92m",
        "yellow"         => "\x1b[33m",
        "bright_yellow"  => "\x1b[93m",
        "blue"           => "\x1b[34m",
        "bright_blue"    => "\x1b[94m",
        "magenta"        => "\x1b[35m",
        "bright_magenta" => "\x1b[95m",
        "cyan"           => "\x1b[36m",
        "bright_cyan"    => "\x1b[96m",
        "white"          => "\x1b[37m",
        "gray" | "grey"  => "\x1b[90m",
        "black"          => "\x1b[30m",
        _ => {
            tracing::warn!("Unknown color '{color}', falling back to gray");
            "\x1b[90m"
        }
    }
    .to_string()
}

/// Keep old name as alias so other modules compile without changes
#[inline]
pub fn hex_to_ansi(color: &str) -> String {
    resolve(color)
}