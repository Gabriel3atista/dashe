use anyhow::Result;

// ── Public types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct GitInfo {
    pub in_repo: bool,
    pub branch:  Option<String>,
    pub status:  RepoStatus,
    pub ahead:   u32,
    pub behind:  u32,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum RepoStatus { #[default] Clean, Staged, Dirty }

// ── Detection ─────────────────────────────────────────────────────────────────

pub fn detect() -> GitInfo {
    try_detect().unwrap_or_default()
}

fn try_detect() -> Result<GitInfo> {
    let repo = match git2::Repository::discover(".") {
        Ok(r)  => r,
        Err(_) => return Ok(GitInfo::default()),
    };

    let branch  = get_branch_name(&repo);
    let status  = get_repo_status(&repo)?;
    let (ahead, behind) = get_ahead_behind(&repo).unwrap_or((0, 0));

    Ok(GitInfo { in_repo: true, branch, status, ahead, behind })
}

fn get_branch_name(repo: &git2::Repository) -> Option<String> {
    if repo.is_empty().unwrap_or(true) {
        return Some("(empty)".to_string());
    }
    let head = repo.head().ok()?;
    if head.is_branch() {
        return head.shorthand().map(|s| s.to_string());
    }
    let hash = format!("{}", head.peel_to_commit().ok()?.id());
    Some(format!(":{}", &hash[..7]))
}

fn get_repo_status(repo: &git2::Repository) -> Result<RepoStatus> {
    let mut opts = git2::StatusOptions::new();
    opts.include_untracked(true)
        .recurse_untracked_dirs(false)
        .exclude_submodules(true);

    let statuses = repo.statuses(Some(&mut opts))?;
    let (mut dirty, mut staged) = (false, false);

    for e in statuses.iter() {
        let s = e.status();
        if s.intersects(
            git2::Status::INDEX_NEW | git2::Status::INDEX_MODIFIED |
            git2::Status::INDEX_DELETED | git2::Status::INDEX_RENAMED |
            git2::Status::INDEX_TYPECHANGE,
        ) { staged = true; }
        if s.intersects(
            git2::Status::WT_NEW | git2::Status::WT_MODIFIED |
            git2::Status::WT_DELETED | git2::Status::WT_TYPECHANGE |
            git2::Status::WT_RENAMED,
        ) { dirty = true; }
    }

    Ok(if dirty { RepoStatus::Dirty } else if staged { RepoStatus::Staged } else { RepoStatus::Clean })
}

fn get_ahead_behind(repo: &git2::Repository) -> Result<(u32, u32)> {
    let head   = repo.head()?;
    let local  = head.target().ok_or_else(|| anyhow::anyhow!("no target"))?;
    let bname  = head.shorthand().ok_or_else(|| anyhow::anyhow!("no branch"))?;
    let upname = format!("refs/remotes/origin/{bname}");
    let upstream = match repo.find_reference(&upname) {
        Ok(r)  => r.target().ok_or_else(|| anyhow::anyhow!("no upstream target"))?,
        Err(_) => return Ok((0, 0)),
    };
    let (a, b) = repo.graph_ahead_behind(local, upstream)?;
    Ok((a as u32, b as u32))
}

// ── Upstream arrows (used by prompt renderer) ─────────────────────────────────

pub fn upstream_info(info: &GitInfo) -> Option<String> {
    if info.ahead == 0 && info.behind == 0 { return None; }
    let r = "\x1b[0m";
    let mut parts = Vec::new();
    if info.ahead  > 0 { parts.push(format!("\x1b[92m↑{}{r}", info.ahead));  }
    if info.behind > 0 { parts.push(format!("\x1b[91m↓{}{r}", info.behind)); }
    Some(parts.join(" "))
}