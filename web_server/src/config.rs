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

        Ok(Self {
            server_port,
            db_username,
            db_password,
            db_name,
            db_host,
            db_port,
            default_days_limit,
            default_frequency_hours,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use config::Config;

    fn base_builder() -> config::builder::ConfigBuilder<config::builder::DefaultState> {
        Config::builder()
            .set_override("db_username", "user")
            .unwrap()
            .set_override("db_password", "pass")
            .unwrap()
            .set_override("db_name", "yscraper")
            .unwrap()
            .set_override("db_host", "localhost")
            .unwrap()
    }

    #[test]
    fn happy_path_fields_map_correctly() {
        let cfg = base_builder()
            .set_override("port", 8080)
            .unwrap()
            .set_override("db_port", 5433)
            .unwrap()
            .set_override("default_days_limit", 14)
            .unwrap()
            .set_override("default_frequency_hours", 12)
            .unwrap()
            .build()
            .unwrap();

        let result = AppConfig::from_config(&cfg).unwrap();

        assert_eq!(result.server_port, 8080);
        assert_eq!(result.db_username, "user");
        assert_eq!(result.db_password, "pass");
        assert_eq!(result.db_name, "yscraper");
        assert_eq!(result.db_host, "localhost");
        assert_eq!(result.db_port, 5433);
        assert_eq!(result.default_days_limit, 14);
        assert_eq!(result.default_frequency_hours, 12);
    }

    #[test]
    fn defaults_applied_when_optional_fields_omitted() {
        let cfg = base_builder().build().unwrap();

        let result = AppConfig::from_config(&cfg).unwrap();

        assert_eq!(result.server_port, 3000);
        assert_eq!(result.db_port, 5432);
        assert_eq!(result.default_days_limit, 7);
        assert_eq!(result.default_frequency_hours, 24);
    }

    #[test]
    fn missing_required_field_returns_err() {
        let cfg = Config::builder()
            .set_override("db_password", "pass")
            .unwrap()
            .set_override("db_name", "yscraper")
            .unwrap()
            .set_override("db_host", "localhost")
            .unwrap()
            .build()
            .unwrap();

        let result = AppConfig::from_config(&cfg);

        assert!(matches!(result, Err(config::ConfigError::NotFound(_))));
    }
}
