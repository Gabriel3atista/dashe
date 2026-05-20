use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::config::Config;

const SERVICE: &str = "dashe";
const ACCOUNT: &str = "token";
const DEFAULT_ENDPOINT: &str = "https://api.dashe.dev/v1";

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
struct SyncPayload {
    config: String, // TOML string
    version: u32,
}

#[derive(Debug, Deserialize)]
struct SyncResponse {
    ok: bool,
    message: Option<String>,
    config: Option<String>,
}

// ── SyncClient ────────────────────────────────────────────────────────────────

pub struct SyncClient {
    endpoint: String,
}

impl SyncClient {
    pub fn new() -> Result<Self> {
        let config = Config::load()?;
        let endpoint = config
            .sync
            .endpoint
            .unwrap_or_else(|| DEFAULT_ENDPOINT.to_string());
        Ok(Self { endpoint })
    }

    fn get_token(&self) -> Option<String> {
        keyring::Entry::new(SERVICE, ACCOUNT)
            .ok()
            .and_then(|e| e.get_password().ok())
    }

    fn set_token(&self, token: &str) -> Result<()> {
        let entry = keyring::Entry::new(SERVICE, ACCOUNT)?;
        entry.set_password(token)?;
        Ok(())
    }

    fn delete_token(&self) -> Result<()> {
        let entry = keyring::Entry::new(SERVICE, ACCOUNT)?;
        let _ = entry.delete_password();
        Ok(())
    }

    pub async fn login(&self) -> Result<()> {
        println!("\x1b[96m🔑 Dashe Cloud Login\x1b[0m\n");

        let methods = vec!["Email / Password", "GitHub OAuth"];
        let choice = dialoguer::Select::new()
            .with_prompt("Login method")
            .items(&methods)
            .default(0)
            .interact()?;

        match choice {
            0 => self.login_email().await,
            1 => self.login_github().await,
            _ => unreachable!(),
        }
    }

    async fn login_email(&self) -> Result<()> {
        let email: String = dialoguer::Input::new()
            .with_prompt("Email")
            .interact_text()?;

        let password = dialoguer::Password::new()
            .with_prompt("Password")
            .interact()?;

        let client = reqwest::Client::new();
        let res = client
            .post(format!("{}/auth/login", self.endpoint))
            .json(&serde_json::json!({ "email": email, "password": password }))
            .send()
            .await;

        match res {
            Ok(r) if r.status().is_success() => {
                let body: serde_json::Value = r.json().await?;
                if let Some(token) = body.get("token").and_then(|t| t.as_str()) {
                    self.set_token(token)?;
                    println!("\n\x1b[92m✅ Logged in as {email}\x1b[0m");
                } else {
                    anyhow::bail!("Unexpected response from server");
                }
            }
            Ok(r) => {
                anyhow::bail!("Login failed: HTTP {}", r.status());
            }
            Err(e) => {
                println!("\x1b[93m⚠️  Could not reach Dashe cloud: {e}\x1b[0m");
                println!("Continuing in offline mode.");
            }
        }
        Ok(())
    }

    async fn login_github(&self) -> Result<()> {
        println!("\nOpen this URL in your browser:");
        println!("\x1b[96m  {}/auth/github\x1b[0m", self.endpoint);
        println!("\nPaste the token shown after authorizing:\n");

        let token: String = dialoguer::Input::new()
            .with_prompt("Token")
            .interact_text()?;

        self.set_token(&token)?;
        println!("\n\x1b[92m✅ Logged in via GitHub\x1b[0m");
        Ok(())
    }

    pub fn logout(&self) -> Result<()> {
        self.delete_token()?;
        println!("\x1b[92m✅ Logged out. Credentials removed.\x1b[0m");
        Ok(())
    }

    pub async fn push(&self) -> Result<()> {
        let token = self
            .get_token()
            .ok_or_else(|| anyhow::anyhow!("Not logged in. Run `dashe sync login`."))?;

        let config = Config::load()?;
        let config_str = toml::to_string_pretty(&config)?;

        let payload = SyncPayload {
            config: config_str,
            version: 1,
        };

        let client = reqwest::Client::new();
        let res = client
            .post(format!("{}/config/push", self.endpoint))
            .bearer_auth(&token)
            .json(&payload)
            .send()
            .await
            .context("Failed to reach Dashe cloud")?;

        if res.status().is_success() {
            println!("\x1b[92m✅ Config pushed to cloud\x1b[0m");
        } else {
            anyhow::bail!("Push failed: HTTP {}", res.status());
        }
        Ok(())
    }

    pub async fn pull(&self) -> Result<()> {
        let token = self
            .get_token()
            .ok_or_else(|| anyhow::anyhow!("Not logged in. Run `dashe sync login`."))?;

        let client = reqwest::Client::new();
        let res = client
            .get(format!("{}/config/pull", self.endpoint))
            .bearer_auth(&token)
            .send()
            .await
            .context("Failed to reach Dashe cloud")?;

        if res.status().is_success() {
            let body: SyncResponse = res.json().await?;
            if let Some(config_str) = body.config {
                let config: Config = toml::from_str(&config_str)
                    .context("Invalid config received from cloud")?;

                // Backup current
                let backup = Config::config_path().with_extension("toml.bak");
                if Config::config_path().exists() {
                    std::fs::copy(Config::config_path(), &backup)?;
                    println!("  Backed up existing config to {}", backup.display());
                }

                config.save()?;
                println!("\x1b[92m✅ Config pulled and applied\x1b[0m");
            }
        } else {
            anyhow::bail!("Pull failed: HTTP {}", res.status());
        }
        Ok(())
    }

    pub fn status(&self) -> Result<()> {
        let config = Config::load()?;
        println!("\n\x1b[96m☁️  Sync Status\x1b[0m\n");

        let logged_in = self.get_token().is_some();
        println!(
            "  Enabled:    {}",
            if config.sync.enabled { "\x1b[92mYes\x1b[0m" } else { "\x1b[90mNo\x1b[0m" }
        );
        println!(
            "  Logged in:  {}",
            if logged_in { "\x1b[92mYes\x1b[0m" } else { "\x1b[91mNo\x1b[0m" }
        );
        println!("  Endpoint:   {}", config.sync.endpoint.as_deref().unwrap_or(DEFAULT_ENDPOINT));

        if !logged_in {
            println!("\n  Run `dashe sync login` to authenticate.");
        }
        println!();
        Ok(())
    }
}