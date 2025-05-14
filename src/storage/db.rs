//! SQLite-based internal database for persistent storage.
//!
//! Provides functionality for storing user credentials and other
//! data that needs to persist between server restarts.

use std::{io::ErrorKind, sync::Arc, time::SystemTime};

use chrono::{DateTime, Utc};
use log::{error, info, warn};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;
use sha3::{Digest, Keccak256};
use uuid::Uuid;

use crate::utils::settings::Settings;

/// Internal database for persistent storage.
///
/// Manages a SQLite database for storing user credentials and other persistent data.
#[derive(Clone, Debug)]
pub struct InternalDB {
  /// Path to the SQLite database file
  pub path: String,
  /// Path for database backups
  pub backup_path: String,
  /// Interval between automatic backups in seconds
  pub backup_interval: u64,
  /// Connection pool for the SQLite database
  pub pool: Arc<r2d2::Pool<SqliteConnectionManager>>,
}

impl InternalDB {
  /// Creates a new internal database instance.
  ///
  /// Initializes the database, creates required tables, and sets up default users
  /// based on the provided settings.
  ///
  /// # Arguments
  ///
  /// * `settings` - Application settings containing database configuration
  ///
  /// # Returns
  ///
  /// A new InternalDB instance connected to the SQLite database.
  pub fn new(settings: &Settings) -> Self {
    let path = settings.get::<String>("server.db.path").unwrap_or_else(|| {
      warn!("No path specified, using default");
      "./.db/internal".to_string()
    });
    let backup_path = settings
      .get::<String>("server.db.backup_path")
      .unwrap_or_else(|| {
        warn!("No backup path specified, using default");
        "./.db/backup".to_string()
      });

    let backup_interval = settings
      .get::<u64>("server.db.backup_interval")
      .unwrap_or_else(|| {
        warn!("No backup interval specified, using default");
        3600
      });

    // Create the db folder and the files if they don't exist
    warn!("Creating main db file: {}", path);
    Self::create_dir(&path);

    warn!("Creating backup db file: {}", backup_path);
    Self::create_dir(&backup_path);

    Self::create_file(format!("{}/db.sqlite3", path).as_str());
    Self::create_file(format!("{}/db.sqlite3", backup_path).as_str());

    let manager = SqliteConnectionManager::file(format!("{}/db.sqlite3", path).as_str());
    let pool = Arc::new(r2d2::Pool::new(manager).unwrap());

    // Create the tables and initialize the database
    Self::create_table(&pool);
    Self::create_user(&pool, &settings);

    Self {
      backup_interval,
      path,
      backup_path,
      pool,
    }
  }

  /// Creates a file if it doesn't exist.
  ///
  /// # Arguments
  ///
  /// * `path` - Path to the file to create
  fn create_file(path: &str) {
    match std::fs::File::create_new(path) {
      Ok(_) => warn!("File created: {}", path),
      Err(e) => {
        if e.kind() == ErrorKind::AlreadyExists {
          info!("File already exists (harmless): {}", path);
        } else {
          error!("Failed to create file '{}': {}", path, e);
        }
      }
    }
  }

  /// Creates a directory and all parent directories if they don't exist.
  ///
  /// # Arguments
  ///
  /// * `path` - Path to the directory to create
  fn create_dir(path: &str) {
    match std::fs::create_dir_all(path) {
      Ok(_) => info!("Already existed: {}", path),
      Err(e) => error!("Failed to create '{}': {}", path, e),
    }
  }

  /// Creates default users based on settings.
  ///
  /// Creates a root user and a regular user with credentials from settings.
  /// If users already exist, it leaves them unchanged.
  ///
  /// # Arguments
  ///
  /// * `pool` - Database connection pool
  /// * `settings` - Application settings containing user credentials
  fn create_user(pool: &Arc<r2d2::Pool<SqliteConnectionManager>>, settings: &Settings) {
    let conn = pool.get().expect("Failed to get connection");

    // Create the id and get the details for the root user
    let id = Uuid::new_v4();
    let root_username = settings.get("server.network.root_user").unwrap_or_else(|| {
      warn!("No root user specified, using default username = root");
      "root".to_string()
    });
    let root_password = settings
      .get("server.network.root_password")
      .unwrap_or_else(|| {
        warn!("No root password specified, using default password = password");
        "password".to_string()
      });

    // Hash the root user password to store in the database
    let mut hasher = Keccak256::new();
    hasher.update(root_password.as_bytes());
    let root_password_hash = hasher.finalize();
    let root_password_hash = format!("{:x}", root_password_hash);

    let time_stamp: DateTime<Utc> = SystemTime::now().into();
    let time_stamp = time_stamp.to_rfc3339();

    // Create the root user
    match conn.execute(
      "INSERT INTO users (id, username, password, created_at, updated_at, root_user) VALUES (?, ?, ?, ?, ?, ?);",
      params![id.to_string(), root_username, root_password_hash, time_stamp, time_stamp, 1],
    ) {
      Ok(_) => warn!("Root user created: {}", root_username),
      Err(e) => {
        if e.to_string().contains("UNIQUE constraint failed") {
          info!("Root user already exists (harmless): {}", root_username);
        } else {
          error!("Failed to create root user '{}': {}", root_username, e);
        }
      }
    }

    // Get the details for the regular user
    let id = Uuid::new_v4();
    let user_name = settings.get("server.network.user").unwrap_or_else(|| {
      warn!("No user name specified, using default username = user");
      "user".to_string()
    });
    let password = settings.get("server.network.password").unwrap_or_else(|| {
      warn!("No password specified, using default password = password");
      "password".to_string()
    });

    // Hash the user password to store in the database
    let mut hasher = Keccak256::new();
    hasher.reset();
    hasher.update(password.as_bytes());
    let password_hash = hasher.finalize();
    let password_hash = format!("{:x}", password_hash);

    // Create the regular user
    match conn.execute(
      "INSERT INTO users (id, username, password, created_at, updated_at, root_user) VALUES (?, ?, ?, ?, ?, ?);",
      params![id.to_string(), user_name, password_hash, time_stamp, time_stamp, 0],
    ) {
      Ok(_) => warn!("User created: {}", user_name),
      Err(e) => {
        if e.to_string().contains("UNIQUE constraint failed") {
          info!("User already exists (harmless): {}", user_name);
        } else {
          error!("Failed to create user '{}': {}", user_name, e);
        }
      }
    }
  }

  /// Creates the required database tables if they don't exist.
  ///
  /// # Arguments
  ///
  /// * `pool` - Database connection pool
  fn create_table(pool: &Arc<r2d2::Pool<SqliteConnectionManager>>) {
    let conn = pool.get().expect("Failed to get connection");
    match conn.execute(
      "CREATE TABLE IF NOT EXISTS users (
        id TEXT PRIMARY KEY NOT NULL,
        username TEXT NOT NULL UNIQUE,
        password TEXT NOT NULL,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL,
        root_user BOOLEAN NOT NULL DEFAULT 0
      );",
      [],
    ) {
      Ok(_) => warn!("Users table created"),
      Err(e) => {
        if e.to_string().contains("already exists") {
          info!("Users table already exists (harmless)");
        } else {
          error!("Failed to create users table: {}", e);
        }
      }
    }
  }
}
