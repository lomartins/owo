use anyhow::Result;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub bind_addr: String,
    pub allow_registration: bool,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite://owo.db?mode=rwc".to_string()),
            bind_addr: std::env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".to_string()),
            allow_registration: std::env::var("OWO_ALLOW_REGISTRATION")
                .map(|v| v != "false" && v != "0")
                .unwrap_or(true),
        })
    }
}
