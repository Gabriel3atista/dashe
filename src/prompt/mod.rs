use anyhow::Result;
use std::env;
use std::path::PathBuf;

use crate::config::Config;
use crate::git;

// ── Renderer ─────────────────────────────────────────────────────────────────

pub struct Renderer {
    config: Config,
}

impl Renderer {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub async fn render(&self) -> Result<String> {
        let cfg = &self.config;
        let reset = "\x1b[0m";

        // Gather segment data
        let username = env::var("USER").unwrap_or_else(|_| "user".to_string());
        let hostname = hostname();
        let dir = current_dir_display(&cfg.segments.max_dir_depth, cfg.segments.shorten_dir);
        let git = if cfg.git.enabled {
            git::detect()
        } else {
            git::GitInfo::default()
        };

        let mut line1 = String::new();

        for (i, segment) in cfg.prompt.layout.iter().enumerate() {
            let part = match segment.as_str() {
                "avatar" => {
                    if let Some(av) = &cfg.prompt.avatar {
                        format!("{av} ")
                    } else {
                        String::new()
                    }
                }
                "user" => {
                    let color = hex_to_ansi(&cfg.colors.username);
                    format!("{color}{username}{reset}")
                }
                "hostname" => {
                    let color = hex_to_ansi(&cfg.colors.hostname);
                    let sep_color = hex_to_ansi(&cfg.colors.separator);
                    format!("{sep_color}@{reset}{color}{hostname}{reset}")
                }
                "dir" => {
                    let color = hex_to_ansi(&cfg.colors.directory);
                    format!(" {color}{dir}{reset}")
                }
                "git_branch" => {
                    if git.in_repo {
                        if let Some(branch) = &git.branch {
                            let color = git::branch_color(branch, &cfg.git);
                            format!(" {color}{branch}{reset}")
                        } else {
                            String::new()
                        }
                    } else {
                        String::new()
                    }
                }
                "git_status" => {
                    if git.in_repo {
                        let indicator = git::status_indicator(&git, &cfg.git);
                        // Show upstream info if any
                        let upstream = git::upstream_info(&git)
                            .map(|u| format!(" {u}"))
                            .unwrap_or_default();
                        format!(" {indicator}{upstream}")
                    } else {
                        String::new()
                    }
                }
                "time" => {
                    if cfg.prompt.show_time {
                        let now = chrono::Local::now();
                        let formatted = now.format(&cfg.prompt.time_format).to_string();
                        let sep_color = hex_to_ansi(&cfg.colors.separator);
                        format!(" {sep_color}[{formatted}]{reset}")
                    } else {
                        String::new()
                    }
                }
                "python_venv" => {
                    if cfg.segments.show_python_venv {
                        if let Ok(venv) = env::var("VIRTUAL_ENV") {
                            let name = PathBuf::from(venv)
                                .file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_default();
                            format!(" \x1b[33m(🐍 {name}){reset}")
                        } else {
                            String::new()
                        }
                    } else {
                        String::new()
                    }
                }
                "node_version" => {
                    if cfg.segments.show_node_version {
                        node_version()
                            .map(|v| format!(" \x1b[32m(⬡ {v}){reset}"))
                            .unwrap_or_default()
                    } else {
                        String::new()
                    }
                }
                "exit_code" => {
                    // Exit code is passed via DASHE_EXIT env var from PROMPT_COMMAND
                    if cfg.segments.show_exit_code {
                        if let Ok(code) = env::var("DASHE_EXIT") {
                            if code != "0" && !code.is_empty() {
                                format!(" \x1b[91m✘ {code}{reset}")
                            } else {
                                String::new()
                            }
                        } else {
                            String::new()
                        }
                    } else {
                        String::new()
                    }
                }
                _ => String::new(),
            };

            if !part.is_empty() && i > 0 && segment != "hostname" {
                // hostname already adds its own separator
                line1.push_str(&part);
            } else {
                line1.push_str(&part);
            }
        }

        // Build final PS1
        let sym_color = hex_to_ansi(&cfg.colors.prompt_symbol);
        let symbol = &cfg.prompt.symbol;

        let output = if cfg.prompt.two_lines {
            format!("{line1}\n{sym_color}{symbol}{reset} ")
        } else {
            format!("{line1} {sym_color}{symbol}{reset} ")
        };

        Ok(output)
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn hostname() -> String {
    std::fs::read_to_string("/etc/hostname")
        .map(|s| s.trim().to_string())
        .or_else(|_| env::var("HOSTNAME"))
        .unwrap_or_else(|_| "localhost".to_string())
}

fn current_dir_display(max_depth: &usize, shorten: bool) -> String {
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/home"));

    // Replace home prefix with ~
    let display = if let Ok(rel) = cwd.strip_prefix(&home) {
        if rel.as_os_str().is_empty() {
            "~".to_string()
        } else {
            format!("~/{}", rel.display())
        }
    } else {
        cwd.display().to_string()
    };

    if shorten && *max_depth > 0 {
        shorten_path(&display, *max_depth)
    } else {
        display
    }
}

fn shorten_path(path: &str, max_depth: usize) -> String {
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() <= max_depth + 1 {
        return path.to_string();
    }

    let visible = &parts[parts.len() - max_depth..];
    let prefix = if path.starts_with('~') { "~" } else { "" };
    format!("{prefix}/…/{}", visible.join("/"))
}

fn node_version() -> Option<String> {
    // Check .nvmrc or .node-version in current dir first
    if let Ok(nvmrc) = std::fs::read_to_string(".nvmrc") {
        return Some(nvmrc.trim().to_string());
    }

    // Try running node --version
    std::process::Command::new("node")
        .arg("--version")
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                String::from_utf8(o.stdout)
                    .ok()
                    .map(|s| s.trim().trim_start_matches('v').to_string())
            } else {
                None
            }
        })
}

/// Convert a HEX color like "#3fb950" to ANSI RGB escape
pub fn hex_to_ansi(hex: &str) -> String {
    let hex = hex.trim_start_matches('#');

    // Named color fallback
    if hex.len() != 6 {
        return named_color_to_ansi(hex);
    }

    if let (Ok(r), Ok(g), Ok(b)) = (
        u8::from_str_radix(&hex[0..2], 16),
        u8::from_str_radix(&hex[2..4], 16),
        u8::from_str_radix(&hex[4..6], 16),
    ) {
        format!("\x1b[38;2;{r};{g};{b}m")
    } else {
        "\x1b[0m".to_string()
    }
}

fn named_color_to_ansi(name: &str) -> String {
    match name {
        "red" => "\x1b[31m",
        "green" => "\x1b[32m",
        "yellow" => "\x1b[33m",
        "blue" => "\x1b[34m",
        "magenta" => "\x1b[35m",
        "cyan" => "\x1b[36m",
        "white" => "\x1b[37m",
        "gray" | "grey" => "\x1b[90m",
        _ => "\x1b[0m",
    }
    .to_string()
}
