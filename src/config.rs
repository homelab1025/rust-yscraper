use config::Config;
use serde::Deserialize;

/// Supported keys (flat or namespaced):
/// - `port` or `server.port` (u16) — HTTP server port; default: 3000
/// - `db_path` or `db.path` (string) — SQLite database file path; default: "comments.db"
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct AppConfig {
    pub server_port: u16,
    pub db_path: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server_port: 3000,
            db_path: "comments.db".to_string(),
        }
    }
}

impl AppConfig {
    /// Build `AppConfig` from an existing `config::Config`. Useful for tests
    /// where the source can be in-memory; no disk access required.
    pub fn from_config(cfg: &Config) -> Self {
        let mut out = Self::default();
        out.server_port = cfg
            .get_int("port")
            .or(cfg.get_int("server.port"))
            .unwrap_or_default() as u16;
        out.db_path = cfg
            .get_string("db_path")
            .or(cfg.get_string("db.path"))
            .unwrap_or_default();

        out
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
        assert!(!cfg.db_path.is_empty(), "default db_path should be non-empty");
    }

    #[test]
    fn from_config_reads_flat_keys() {
        let cfg = config::Config::builder()
            .set_override("port", 4321).unwrap()
            .set_override("db_path", "flat.db").unwrap()
            .build().unwrap();

        let app = AppConfig::from_config(&cfg);
        assert_eq!(app.server_port, 4321);
        assert_eq!(app.db_path, "flat.db");
    }

    #[test]
    fn from_config_reads_namespaced_keys() {
        let cfg = config::Config::builder()
            .set_override("server.port", 5555).unwrap()
            .set_override("db.path", "ns.db").unwrap()
            .build().unwrap();

        let app = AppConfig::from_config(&cfg);
        assert_eq!(app.server_port, 5555);
        assert_eq!(app.db_path, "ns.db");
    }

    #[test]
    fn flat_keys_take_precedence_over_namespaced() {
        let cfg = config::Config::builder()
            .set_override("server.port", 1111).unwrap()
            .set_override("db.path", "ns.db").unwrap()
            .set_override("port", 2222).unwrap()
            .set_override("db_path", "flat.db").unwrap()
            .build().unwrap();

        let app = AppConfig::from_config(&cfg);
        assert_eq!(app.server_port, 2222);
        assert_eq!(app.db_path, "flat.db");
    }
}
