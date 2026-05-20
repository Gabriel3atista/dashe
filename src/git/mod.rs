use anyhow::Result;
use glob::Pattern;

use crate::config::GitConfig;

// ── Public types ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct GitInfo {
    pub in_repo: bool,
    pub branch: Option<String>,
    pub status: RepoStatus,
    pub ahead: u32,
    pub behind: u32,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum RepoStatus {
    #[default]
    Clean,
    Staged,
    Dirty,
}

// ── Detection ────────────────────────────────────────────────────────────────

pub fn detect() -> GitInfo {
    match try_detect() {
        Ok(info) => info,
        Err(_) => GitInfo::default(),
    }
}

fn try_detect() -> Result<GitInfo> {
    let repo = match git2::Repository::discover(".") {
        Ok(r) => r,
        Err(_) => return Ok(GitInfo::default()),
    };

    // Branch name
    let branch = get_branch_name(&repo);

    // Status
    let status = get_repo_status(&repo)?;

    // Ahead/behind
    let (ahead, behind) = get_ahead_behind(&repo).unwrap_or((0, 0));

    Ok(GitInfo {
        in_repo: true,
        branch,
        status,
        ahead,
        behind,
    })
}

fn get_branch_name(repo: &git2::Repository) -> Option<String> {
    if repo.is_empty().unwrap_or(true) {
        return Some("(empty)".to_string());
    }

    // Try HEAD symbolic ref first
    if let Ok(head) = repo.head() {
        if head.is_branch() {
            return head.shorthand().map(|s| s.to_string());
        }
        // Detached HEAD — show short commit hash
        if let Ok(commit) = head.peel_to_commit() {
            let hash = format!("{}", commit.id());
            return Some(format!(":{}", &hash[..7]));
        }
    }
    None
}

fn get_repo_status(repo: &git2::Repository) -> Result<RepoStatus> {
    let mut opts = git2::StatusOptions::new();
    opts.include_untracked(true)
        .recurse_untracked_dirs(false)
        .exclude_submodules(true);

    let statuses = repo.statuses(Some(&mut opts))?;

    let mut has_dirty = false;
    let mut has_staged = false;

    for entry in statuses.iter() {
        let s = entry.status();

        // Staged changes
        if s.intersects(
            git2::Status::INDEX_NEW
                | git2::Status::INDEX_MODIFIED
                | git2::Status::INDEX_DELETED
                | git2::Status::INDEX_RENAMED
                | git2::Status::INDEX_TYPECHANGE,
        ) {
            has_staged = true;
        }

        // Unstaged / untracked changes
        if s.intersects(
            git2::Status::WT_NEW
                | git2::Status::WT_MODIFIED
                | git2::Status::WT_DELETED
                | git2::Status::WT_TYPECHANGE
                | git2::Status::WT_RENAMED,
        ) {
            has_dirty = true;
        }
    }

    Ok(if has_dirty {
        RepoStatus::Dirty
    } else if has_staged {
        RepoStatus::Staged
    } else {
        RepoStatus::Clean
    })
}

fn get_ahead_behind(repo: &git2::Repository) -> Result<(u32, u32)> {
    let head = repo.head()?;
    let local = head.target().ok_or_else(|| anyhow::anyhow!("no target"))?;

    let branch_name = head
        .shorthand()
        .ok_or_else(|| anyhow::anyhow!("no branch name"))?;
    let upstream_name = format!("refs/remotes/origin/{branch_name}");

    let upstream = match repo.find_reference(&upstream_name) {
        Ok(r) => r.target().ok_or_else(|| anyhow::anyhow!("no upstream target"))?,
        Err(_) => return Ok((0, 0)),
    };

    let (ahead, behind) = repo.graph_ahead_behind(local, upstream)?;
    Ok((ahead as u32, behind as u32))
}

// ── Branch color resolution ───────────────────────────────────────────────────

pub fn branch_color(branch: &str, cfg: &GitConfig) -> &'static str {
    // Check explicit rules first (with glob support)
    for (pattern, color) in &cfg.branch_rules {
        if pattern == branch {
            return resolve_color(color);
        }
        // Glob match
        if let Ok(pat) = Pattern::new(pattern) {
            if pat.matches(branch) {
                return resolve_color(color);
            }
        }
    }

    // Defaults: main/master = red, everything else = yellow
    match branch {
        "main" | "master" => "\x1b[31m",
        _ => "\x1b[33m",
    }
}

fn resolve_color(name: &str) -> &'static str {
    match name {
        "red" => "\x1b[31m",
        "bright_red" => "\x1b[91m",
        "yellow" => "\x1b[33m",
        "bright_yellow" => "\x1b[93m",
        "green" => "\x1b[32m",
        "bright_green" => "\x1b[92m",
        "blue" => "\x1b[34m",
        "bright_blue" => "\x1b[94m",
        "cyan" => "\x1b[36m",
        "bright_cyan" => "\x1b[96m",
        "magenta" => "\x1b[35m",
        "bright_magenta" => "\x1b[95m",
        "white" => "\x1b[37m",
        "gray" | "grey" => "\x1b[90m",
        _ => {
            // Could be HEX — fallback to yellow for now
            "\x1b[33m"
        }
    }
}

// ── Status indicator renderer ─────────────────────────────────────────────────

pub fn status_indicator(info: &GitInfo, cfg: &GitConfig) -> String {
    let reset = "\x1b[0m";

    match info.status {
        RepoStatus::Dirty => {
            let icon = &cfg.status.dirty;
            format!("\x1b[31m{icon}{reset}")
        }
        RepoStatus::Staged => {
            let icon = &cfg.status.staged;
            format!("\x1b[33m{icon}{reset}")
        }
        RepoStatus::Clean => {
            let icon = &cfg.status.clean;
            format!("\x1b[32m{icon}{reset}")
        }
    }
}

// ── Upstream sync display ─────────────────────────────────────────────────────

pub fn upstream_info(info: &GitInfo) -> Option<String> {
    if info.ahead == 0 && info.behind == 0 {
        return None;
    }

    let mut parts = Vec::new();
    if info.ahead > 0 {
        parts.push(format!("\x1b[92m↑{}\x1b[0m", info.ahead));
    }
    if info.behind > 0 {
        parts.push(format!("\x1b[91m↓{}\x1b[0m", info.behind));
    }
    Some(parts.join(" "))
}
