// Integration tests for Dashe core modules
// Run with: cargo test

#[cfg(test)]
mod config_tests {
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn temp_config() -> (TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        (dir, path)
    }

    #[test]
    fn default_config_is_valid() {
        let config = dashe::config::Config::default();
        assert!(!config.prompt.symbol.is_empty());
        assert!(config.git.enabled);
        assert!(!config.prompt.layout.is_empty());
    }

    #[test]
    fn config_serializes_and_deserializes() {
        let config = dashe::config::Config::default();
        let serialized = toml::to_string_pretty(&config).unwrap();
        let deserialized: dashe::config::Config = toml::from_str(&serialized).unwrap();
        assert_eq!(config.prompt.symbol, deserialized.prompt.symbol);
        assert_eq!(config.colors.username, deserialized.colors.username);
    }
}

#[cfg(test)]
mod git_tests {
    #[test]
    fn branch_color_main_is_red() {
        let cfg = dashe::config::GitConfig::default();
        let color = dashe::git::branch_color("main", &cfg);
        assert!(color.contains("31"), "main branch should be red (ANSI 31)");
    }

    #[test]
    fn branch_color_feature_is_yellow() {
        let cfg = dashe::config::GitConfig::default();
        let color = dashe::git::branch_color("feature/my-thing", &cfg);
        assert!(color.contains("33"), "feature/* should be yellow (ANSI 33)");
    }

    #[test]
    fn branch_color_master_is_red() {
        let cfg = dashe::config::GitConfig::default();
        let color = dashe::git::branch_color("master", &cfg);
        assert!(color.contains("31"), "master branch should be red");
    }
}

#[cfg(test)]
mod prompt_tests {
    use dashe::prompt::hex_to_ansi;

    #[test]
    fn hex_to_ansi_valid() {
        let ansi = hex_to_ansi("#3fb950");
        // Should produce RGB escape sequence
        assert!(ansi.contains("38;2;"), "Expected RGB ANSI code");
        assert!(ansi.contains("63;185;80"), "Expected RGB values for #3fb950");
    }

    #[test]
    fn hex_to_ansi_fallback_named() {
        let ansi = hex_to_ansi("red");
        assert!(ansi.contains("31"));
    }
}

#[cfg(test)]
mod theme_tests {
    use dashe::theme::builtin_themes;

    #[test]
    fn all_presets_are_present() {
        let themes = builtin_themes();
        for name in ["p10k", "starship", "ocean", "retro", "minimal"] {
            assert!(themes.contains_key(name), "Missing theme: {name}");
        }
    }

    #[test]
    fn all_presets_have_valid_colors() {
        let themes = builtin_themes();
        for (name, theme) in &themes {
            assert!(
                !theme.colors.username.is_empty(),
                "Theme {name} missing username color"
            );
            assert!(
                !theme.colors.directory.is_empty(),
                "Theme {name} missing directory color"
            );
        }
    }
}
