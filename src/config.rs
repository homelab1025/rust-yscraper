use config::{Config, File, FileFormat};
use serde::Deserialize;

/// Application configuration loaded from `config.properties` (INI format).
///
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
    /// Load configuration from the optional `config.properties` file at `path`.
    /// If the file is missing or invalid, sane defaults are used with best-effort overrides.
    pub fn load_from_file(path: &str) -> Self {
        let cfg = Config::builder()
            .add_source(File::new(path, FileFormat::Ini).required(false))
            .build();

        match cfg {
            Ok(c) => Self::from_config(&c),
            Err(_) => Self::default(),
        }
    }

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
