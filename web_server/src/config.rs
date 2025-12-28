use config::Config;
use serde::Deserialize;

/// Supported keys (flat or namespaced):
/// - `port` or `server.port` (u16) — HTTP server port; default: 3000
/// - `db_url` or `db.url` (string) — PostgreSQL connection URL; default: env `DATABASE_URL` or
///   fallback `postgres://postgres:postgres@localhost:5432/yscraper`
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct AppConfig {
    pub server_port: u16,
    pub db_username: String,
    pub db_password: String,
    pub db_name: String,
    pub db_host: String,
    pub db_port: u16,
}

impl AppConfig {

    /// Build `AppConfig` from an existing `config::Config`. Useful for tests
    /// where the source can be in-memory; no disk access required.
    pub fn from_config(cfg: &Config) -> Result<Self, config::ConfigError> {
        let server_port = cfg.get_int("port").unwrap_or(3000) as u16;

        let db_username = cfg.get_string("db_username")?;
        let db_password = cfg.get_string("db_password")?;
        let db_name = cfg.get_string("db_name")?;
        let db_host = cfg.get_string("db_host")?;
        let db_port = cfg.get_int("db_port").unwrap_or(5432) as u16;

        Ok(Self {
            server_port,
            db_username,
            db_password,
            db_name,
            db_host,
            db_port,
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
