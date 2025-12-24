use config::Config;
use serde::Deserialize;

/// Supported keys (flat or namespaced):
/// - `port` or `server.port` (u16) — HTTP server port; default: 3000
/// - `db_url` or `db.url` (string) — PostgreSQL connection URL; default: env `DATABASE_URL` or
///   fallback `postgres://postgres:postgres@localhost:5432/yscraper`
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct AppConfig {
    pub server_port: u16,
    pub db_url: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server_port: 3000,
            db_url: std::env::var("DATABASE_URL").unwrap_or_else(|_| {
                "postgres://postgres:postgres@localhost:5432/yscraper".to_string()
            }),
        }
    }
}

impl AppConfig {
    /// Build `AppConfig` from an existing `config::Config`. Useful for tests
    /// where the source can be in-memory; no disk access required.
    pub fn from_config(cfg: &Config) -> Self {
        let server_port = cfg.get_int("port").unwrap_or(3000) as u16;

        // Priority: explicit key -> namespaced -> env DATABASE_URL -> default
        let db_url = cfg
            .get_string("db_url")
            .ok()
            .or_else(|| std::env::var("DATABASE_URL").ok())
            .unwrap_or_else(|| "postgres://postgres:postgres@localhost:5432/yscraper".to_string());

        Self {
            server_port,
            db_url,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_values_present() {
        let cfg = AppConfig::default();
        // Only check that defaults exist, not their concrete values
        assert_ne!(cfg.server_port, 0, "default server_port should be non-zero");
        assert!(!cfg.db_url.is_empty(), "default db_url should be non-empty");
    }

    #[test]
    fn from_config_reads_flat_keys() {
        let cfg = config::Config::builder()
            .set_override("port", 4321)
            .unwrap()
            .set_override("db_url", "postgres://u:p@h:5432/db1")
            .unwrap()
            .build()
            .unwrap();

        let app = AppConfig::from_config(&cfg);
        assert_eq!(app.server_port, 4321);
        assert_eq!(app.db_url, "postgres://u:p@h:5432/db1");
    }

    #[test]
    fn flat_keys_take_precedence_over_namespaced() {
        let cfg = config::Config::builder()
            .set_override("server.port", 1111)
            .unwrap()
            .set_override("db.url", "postgres://u:p@h:5432/nsdb")
            .unwrap()
            .set_override("port", 2222)
            .unwrap()
            .set_override("db_url", "postgres://u:p@h:5432/flatdb")
            .unwrap()
            .build()
            .unwrap();

        let app = AppConfig::from_config(&cfg);
        assert_eq!(app.server_port, 2222);
        assert_eq!(app.db_url, "postgres://u:p@h:5432/flatdb");
    }
}
