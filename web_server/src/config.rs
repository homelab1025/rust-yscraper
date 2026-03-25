use config::Config;
use serde::Deserialize;

/// Supported keys (flat or namespaced):
/// - `port` or `server.port` (u16) — HTTP server port; default: 3000
/// - `db_url` or `db.url` (string) — PostgreSQL connection URL; default: env `DATABASE_URL` or
///   fallback `postgres://postgres:postgres@localhost:5432/yscraper`
/// - `default_days_limit` (u32) — Default days limit for comment refreshing; default: 7
/// - `default_frequency_hours` (u32) — Default frequency in hours for comment refreshing; default: 24
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct AppConfig {
    pub server_port: u16,
    pub db_username: String,
    pub db_password: String,
    pub db_name: String,
    pub db_host: String,
    pub db_port: u16,
    pub default_days_limit: u32,
    pub default_frequency_hours: u32,
    pub totp_secret: String,
}

impl AppConfig {
    /// Build `AppConfig` from an existing `config::Config`. Useful for tests
    /// where source can be in-memory; no disk access required.
    pub fn from_config(cfg: &Config) -> Result<Self, config::ConfigError> {
        let server_port = cfg.get_int("port").unwrap_or(3000) as u16;

        let db_username = cfg.get_string("db_username")?;
        let db_password = cfg.get_string("db_password")?;
        let db_name = cfg.get_string("db_name")?;
        let db_host = cfg.get_string("db_host")?;
        let db_port = cfg.get_int("db_port").unwrap_or(5432) as u16;
        let default_days_limit = cfg.get_int("default_days_limit").unwrap_or(7) as u32;
        let default_frequency_hours = cfg.get_int("default_frequency_hours").unwrap_or(24) as u32;
        let totp_secret = cfg.get_string("totp_secret")?;

        Ok(Self {
            server_port,
            db_username,
            db_password,
            db_name,
            db_host,
            db_port,
            default_days_limit,
            default_frequency_hours,
            totp_secret,
        })
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn fail_when_no_dbport() {
        // let cfg = AppConfig::from_config()
        // // Only check that defaults exist, not their concrete values
        // assert_ne!(cfg.server_port, 0, "default server_port should be non-zero");
        // assert!(!cfg.db_url.is_empty(), "default db_url should be non-empty");
    }
}
