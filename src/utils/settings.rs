use config::{self, Config, File};
use log::error;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Settings {
  pub server: Server,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Server {
  pub name: String,
  pub version: String,
  pub description: String,
  pub network: Network,
  pub db: Database,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Network {
  pub host: String,
  pub port: u16,
  pub root_user: String,
  pub root_password: String,
  pub user: String,
  pub password: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Database {
  pub path: String,
  pub backup_path: String,
  pub max_size: u32,
  pub backup_interval: u64,
  pub compression: bool,
  pub enable_logging: bool,
}

impl Settings {
  pub fn new<'a>(filename: impl Into<Option<&'a str>>) -> Self {
    let filename = filename.into();

    // Create a default configuration
    let default_settings = Settings {
      server: Server {
        name: "Default Server".into(),
        version: "1.0".into(),
        description: "A default server configuration".into(),
        network: Network {
          host: "127.0.0.1".into(),
          port: 8080,
          root_user: "root".into(),
          root_password: "rootpassword".into(),
          user: "admin".into(),
          password: "securepassword".into(),
        },
        db: Database {
          path: "db.sqlite".into(),
          backup_path: "backup.sqlite".into(),
          max_size: 1024,
          backup_interval: 3600,
          compression: true,
          enable_logging: true,
        },
      },
    };

    // Determine which config file to load
    let config_file = filename.unwrap_or("config.toml");

    // Try to load configuration from file
    match Config::builder()
      .add_source(File::with_name(config_file).required(false))
      .build()
    {
      Ok(config) => match config.try_deserialize::<Settings>() {
        Ok(settings) => settings,
        Err(e) => {
          error!("Failed to parse config file {}: {}", config_file, e);
          default_settings
        }
      },
      Err(e) => {
        error!("Failed to load config file {}: {}", config_file, e);
        default_settings
      }
    }
  }

  pub fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
    // First, serialize the entire settings object to a serde_json Value
    let settings_value = match serde_json::to_value(self) {
      Ok(value) => value,
      Err(_) => return None,
    };

    // Navigate to the requested key using the path
    let mut current = &settings_value;
    for part in key.split('.') {
      // For each part of the path (e.g., "server.network.host")
      match current.get(part) {
        Some(value) => current = value,
        None => return None, // Path doesn't exist
      }
    }

    // Try to deserialize the found value to the requested type
    match serde_json::from_value::<T>(current.clone()) {
      Ok(typed_value) => Some(typed_value),
      Err(_) => None,
    }
  }
}
