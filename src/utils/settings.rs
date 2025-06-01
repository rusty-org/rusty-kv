//! Configuration management module for the key-value server.
//!
//! This module provides functionality to load, parse, and access server configuration
//! from TOML files, with sensible defaults when configuration is missing.

use config::{self, Config, File};
use log::error;
use serde::{Deserialize, Serialize};

/// Main configuration structure for the server.
///
/// Contains all server settings including network configuration and database settings.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Settings {
  pub server: Server,
}

/// Server-specific configuration settings.
///
/// Contains metadata about the server as well as network and database configurations.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Server {
  /// Name of the server instance
  pub name: String,
  /// Version of the server software
  pub version: String,
  /// Description of the server instance
  pub description: String,
  /// Network-related configuration
  pub network: Network,
  /// Database-related configuration
  pub db: Database,
  /// RDB persistence settings
  pub kdb: KDBSettings,
}

/// Network configuration settings.
///
/// Defines how the server interacts on the network, including host, port, and authentication.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Network {
  /// Host address to bind the server to
  pub host: String,
  /// Port number to listen on
  pub port: u16,
  /// Username for root access
  pub root_user: String,
  /// Password for root access
  pub root_password: String,
  /// Username for regular access
  pub user: String,
  /// Password for regular access
  pub password: String,
}

/// Database configuration settings.
///
/// Contains settings for database storage, backups, and performance options.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Database {
  /// Path to the main database file
  pub path: String,
  /// Path for backup database files
  pub backup_path: String,
  /// Maximum size of the database in MB
  pub max_size: u32,
  /// Interval between automatic backups in seconds
  pub backup_interval: u64,
  /// Whether to enable database compression
  pub compression: bool,
  /// Whether to enable detailed database operation logging
  pub enable_logging: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// Represents whether the persistence layer is enabled or not.
///
/// It will load the RDB data while it boots up if enabled,
/// and periodically save the data to disk.
pub struct KDBSettings {
  /// Path to the database file
  pub path: String,
  /// File name
  pub file_name: String,
  /// Whether to enable RDB persistence
  pub persistence: bool,
  /// Interval for RDB backups in seconds
  pub backup_interval: u64,
}

impl Settings {
  /// Creates a new Settings instance.
  ///
  /// Attempts to load settings from the specified configuration file.
  /// Falls back to default settings if the file cannot be read or parsed.
  ///
  /// # Arguments
  ///
  /// * `filename` - Optional name of the configuration file to load
  ///
  /// # Returns
  ///
  /// A new Settings instance with either loaded or default configuration
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
        kdb: KDBSettings {
          path: "/tmp/rustykv.bak".to_string(),
          file_name: "backup.rdb".to_string(),
          persistence: false,
          backup_interval: 3600, // Default backup interval (in seconds)
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

  /// Gets a typed configuration value from a dot-notation path.
  ///
  /// Allows accessing nested configuration values with dot notation paths like
  /// "server.network.host" and deserializes the value to the requested type.
  ///
  /// # Arguments
  ///
  /// * `key` - The dot-notation path to the desired configuration value
  ///
  /// # Returns
  ///
  /// * `Some(T)` - Successfully retrieved and deserialized value
  /// * `None` - Value not found or type conversion failed
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
