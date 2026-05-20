use anyhow::Result;
use std::process::Command;

pub fn run_checks() -> Result<()> {
    println!("\n\x1b[96m🔬 Dashe Doctor\x1b[0m\n");

    let mut passed = 0u32;
    let mut warnings = 0u32;
    let mut errors = 0u32;

    macro_rules! check {
        ($label:expr, $result:expr) => {
            match $result {
                CheckResult::Ok(msg) => {
                    println!("  \x1b[92m✓\x1b[0m  {:<30} {}", $label, msg);
                    passed += 1;
                }
                CheckResult::Warn(msg) => {
                    println!("  \x1b[93m⚠\x1b[0m  {:<30} {}", $label, msg);
                    warnings += 1;
                }
                CheckResult::Fail(msg) => {
                    println!("  \x1b[91m✗\x1b[0m  {:<30} {}", $label, msg);
                    errors += 1;
                }
            }
        };
    }

    // Config
    check!("Config file", check_config());
    check!("Aliases file", check_aliases());

    // Shell environment
    check!("SHELL variable", check_shell());
    check!("TERM variable", check_term());
    check!("PROMPT_COMMAND", check_prompt_command());
    check!("Unicode support", check_unicode());

    // Git
    check!("Git binary", check_git_binary());
    check!("git2 library", check_git2());

    // Fonts
    check!("Nerd Fonts", check_nerd_fonts());

    // Network (optional)
    check!("Internet (sync)", check_network());

    // Performance
    check!("Startup time", check_startup_time());

    println!();
    println!("  Results: \x1b[92m{passed} passed\x1b[0m, \x1b[93m{warnings} warnings\x1b[0m, \x1b[91m{errors} errors\x1b[0m");

    if errors == 0 && warnings == 0 {
        println!("\n  \x1b[92m✅ Everything looks great!\x1b[0m");
    } else if errors == 0 {
        println!("\n  \x1b[93m⚠️  Minor issues found. Dashe should still work fine.\x1b[0m");
    } else {
        println!("\n  \x1b[91m❌ Issues found. Run `dashe init` to reconfigure.\x1b[0m");
    }
    println!();

    Ok(())
}

enum CheckResult {
    Ok(String),
    Warn(String),
    Fail(String),
}

fn check_config() -> CheckResult {
    let path = crate::config::Config::config_path();
    if path.exists() {
        match crate::config::Config::load() {
            Ok(_) => CheckResult::Ok(path.display().to_string()),
            Err(e) => CheckResult::Fail(format!("Parse error: {e}")),
        }
    } else {
        CheckResult::Warn("Not found — run `dashe init`".to_string())
    }
}

fn check_aliases() -> CheckResult {
    let path = crate::config::Config::config_dir().join("aliases.toml");
    if path.exists() {
        CheckResult::Ok("aliases.toml found".to_string())
    } else {
        CheckResult::Warn("No aliases file yet".to_string())
    }
}

fn check_shell() -> CheckResult {
    match std::env::var("SHELL") {
        Ok(shell) if shell.contains("bash") => CheckResult::Ok(shell),
        Ok(shell) => CheckResult::Warn(format!("{shell} (Dashe targets Bash; other shells coming soon)")),
        Err(_) => CheckResult::Fail("SHELL variable not set".to_string()),
    }
}

fn check_term() -> CheckResult {
    match std::env::var("TERM") {
        Ok(term) => {
            if term.contains("256color") || term.contains("truecolor") || term == "xterm-kitty" {
                CheckResult::Ok(format!("{term} — color support ✓"))
            } else {
                CheckResult::Warn(format!("{term} — colors may be limited"))
            }
        }
        Err(_) => CheckResult::Fail("TERM variable not set".to_string()),
    }
}

fn check_prompt_command() -> CheckResult {
    match std::env::var("PROMPT_COMMAND") {
        Ok(pc) if pc.contains("dashe prompt") => {
            CheckResult::Ok("dashe prompt found in PROMPT_COMMAND".to_string())
        }
        Ok(_) => CheckResult::Warn("PROMPT_COMMAND doesn't include `dashe prompt`".to_string()),
        Err(_) => CheckResult::Fail("PROMPT_COMMAND not set — add to ~/.bashrc".to_string()),
    }
}

fn check_unicode() -> CheckResult {
    match std::env::var("LANG") {
        Ok(lang) if lang.to_uppercase().contains("UTF-8") || lang.to_uppercase().contains("UTF8") => {
            CheckResult::Ok(lang)
        }
        Ok(lang) => CheckResult::Warn(format!("{lang} — set LANG=en_US.UTF-8 for emoji support")),
        Err(_) => CheckResult::Warn("LANG not set — emojis may not render".to_string()),
    }
}

fn check_git_binary() -> CheckResult {
    match Command::new("git").arg("--version").output() {
        Ok(out) if out.status.success() => {
            let ver = String::from_utf8_lossy(&out.stdout);
            CheckResult::Ok(ver.trim().to_string())
        }
        _ => CheckResult::Fail("git not found in PATH".to_string()),
    }
}

fn check_git2() -> CheckResult {
    // If we compiled with git2, it's available
    CheckResult::Ok("libgit2 linked statically".to_string())
}

fn check_nerd_fonts() -> CheckResult {
    // Heuristic: check if a known Nerd Font is installed
    let font_dirs = vec![
        std::path::PathBuf::from("/usr/share/fonts"),
        dirs::home_dir()
            .unwrap_or_default()
            .join(".local/share/fonts"),
    ];

    for dir in font_dirs {
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_lowercase();
                if name.contains("nerd") || name.contains("nf-") {
                    return CheckResult::Ok(format!("Found in {}", dir.display()));
                }
            }
        }
    }

    CheckResult::Warn("No Nerd Fonts detected — icons may not render. Get them at nerdfonts.com".to_string())
}

fn check_network() -> CheckResult {
    // Try a simple DNS lookup via curl (non-blocking, 2s timeout)
    match Command::new("curl")
        .args(["-s", "--max-time", "2", "-o", "/dev/null", "-w", "%{http_code}", "https://1.1.1.1"])
        .output()
    {
        Ok(out) => {
            let code = String::from_utf8_lossy(&out.stdout);
            if code.starts_with('2') || code.starts_with('3') {
                CheckResult::Ok("Reachable".to_string())
            } else {
                CheckResult::Warn("Offline — sync disabled".to_string())
            }
        }
        _ => CheckResult::Warn("Cannot check — curl not available".to_string()),
    }
}

fn check_startup_time() -> CheckResult {
    let start = std::time::Instant::now();
    let _ = crate::config::Config::load();
    let elapsed = start.elapsed();
    let ms = elapsed.as_millis();

    if ms < 10 {
        CheckResult::Ok(format!("{ms}ms — excellent"))
    } else if ms < 50 {
        CheckResult::Warn(format!("{ms}ms — acceptable but check disk speed"))
    } else {
        CheckResult::Fail(format!("{ms}ms — slow! Check ~/.config/dashe for issues"))
    }
}
