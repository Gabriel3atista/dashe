# ⚡ Dashe

**Modern terminal customizer for Linux/Bash** — inspired by Starship, Oh My Zsh and Powerlevel10k, with its own identity, blazing performance, and simple configuration.

```
██████╗  █████╗ ███████╗██╗  ██╗███████╗
██╔══██╗██╔══██╗██╔════╝██║  ██║██╔════╝
██║  ██║███████║███████╗███████║█████╗  
██║  ██║██╔══██║╚════██║██╔══██║██╔══╝  
██████╔╝██║  ██║███████║██║  ██║███████╗
╚═════╝ ╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝╚══════╝
```

---

## Features

- 🎨 **Full prompt customization** — layout, colors, icons, separators
- 🌿 **Git integration** — branch coloring, status indicators, ahead/behind
- 📌 **Alias system** — create, list, export/import, categorize
- 🖌️ **Theme presets** — p10k, starship, ocean, retro, minimal
- ☁️ **Cloud sync** — backup config across machines (opt-in)
- 🔬 **Doctor** — diagnose environment issues
- ⚡ **Performance** — startup < 5ms

---

## Installation

```bash
# Build from source
git clone https://github.com/your-username/dashe
cd dashe
cargo build --release

# Copy binary
sudo cp target/release/dashe /usr/local/bin/

# Run wizard
dashe init
```

Add to `~/.bashrc`:

```bash
export PROMPT_COMMAND='DASHE_EXIT=$?; PS1="$(dashe prompt)"'
source ~/.config/dashe/aliases.sh
```

---

## Quick Start

```bash
dashe init                         # Interactive setup wizard
dashe theme set ocean              # Apply ocean theme
dashe alias add gs "git status"    # Add an alias
dashe alias add gp "git push"      # Another alias
dashe doctor                       # Check environment
dashe bench                        # Benchmark startup
```

---

## Configuration

All configuration lives in `~/.config/dashe/config.toml`.

```toml
[prompt]
symbol    = "❯"
avatar    = "🚀"
two_lines = true
layout    = ["avatar", "user", "hostname", "dir", "git_branch", "git_status"]

[colors]
username      = "#3fb950"
hostname      = "#58a6ff"
directory     = "#e6edf3"
git_branch    = "#e3b341"

[git.branch_rules]
main       = "red"
"feature/*" = "yellow"
"hotfix/*"  = "magenta"

[git.status]
dirty  = "●"
staged = "●"
clean  = "●"
```

### Available segments

| Segment | Description |
|---|---|
| `avatar` | Emoji/icon prefix |
| `user` | Username |
| `hostname` | Machine hostname |
| `dir` | Current directory |
| `git_branch` | Branch name with color rules |
| `git_status` | Dirty/staged/clean indicator |
| `time` | Current time |
| `python_venv` | Active virtualenv |
| `node_version` | Node.js version |
| `exit_code` | Last command exit code |

---

## Alias System

```bash
dashe alias add gs "git status"
dashe alias add gp "git push" --category git
dashe alias list
dashe alias list --category git
dashe alias remove gs
dashe alias export my-aliases.toml
dashe alias import my-aliases.toml
```

Aliases are persisted to `~/.config/dashe/aliases.toml` and exported to
`~/.config/dashe/aliases.sh` for sourcing in Bash.

---

## Themes

```bash
dashe theme list
dashe theme set p10k
dashe theme set starship
dashe theme set ocean
dashe theme set retro
dashe theme set minimal
dashe theme current        # Show current colors
dashe theme export theme.toml
dashe theme import theme.toml
```

---

## Git Integration

- **main/master** → Red
- **feature/*** → Yellow
- **hotfix/*** → Magenta
- **Other branches** → Yellow

Status indicators:

| Symbol | Meaning |
|---|---|
| 🔴 ● | Dirty (uncommitted changes) |
| 🟡 ● | Staged (ready to commit) |
| 🟢 ● | Clean |

---

## Cloud Sync

```bash
dashe sync login    # Email or GitHub OAuth
dashe sync push     # Upload config
dashe sync pull     # Download config
dashe sync status
dashe sync logout
```

Works offline — sync is fully opt-in.

---

## Architecture

```
src/
├── main.rs           ← CLI entrypoint (clap)
├── cli/
│   ├── mod.rs        ← Commands enum + dispatch
│   ├── commands.rs   ← Handler functions
│   └── install.rs    ← Wizard + ASCII art
├── config/mod.rs     ← TOML schema + load/save
├── prompt/mod.rs     ← PS1 renderer + segment engine
├── git/mod.rs        ← git2 integration, branch colors, status
├── theme/mod.rs      ← Presets + ThemeManager
├── alias/mod.rs      ← CRUD + shell file generation
├── sync/mod.rs       ← HTTP client + keyring auth
└── doctor/mod.rs     ← Environment diagnostics
```

---

## License

MIT
