use core::hash;
use std::{io::ErrorKind, sync::Arc, time::SystemTime};

use chrono::{DateTime, Utc};
use log::{error, info, warn};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;
use sha3::{Digest, Keccak256};
use uuid::Uuid;

use crate::utils::settings::Settings;

#[derive(Clone, Debug)]
pub struct InternalDB {
  pub path: String,
  pub backup_path: String,
  pub backup_interval: u64,
  pub pool: Arc<r2d2::Pool<SqliteConnectionManager>>,
}

impl InternalDB {
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

    // @INFO Create the tables and initialize the database
    Self::create_table(&pool);
    Self::create_user(&pool, &settings);

    Self {
      backup_interval,
      path,
      backup_path,
      pool,
    }
  }

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

  fn create_dir(path: &str) {
    match std::fs::create_dir_all(path) {
      Ok(_) => info!("Already existed: {}", path),
      Err(e) => error!("Failed to create '{}': {}", path, e),
    }
  }

  fn create_user(pool: &Arc<r2d2::Pool<SqliteConnectionManager>>, settings: &Settings) {
    let conn = pool.get().expect("Failed to get connection");

    // @INFO create the id and get the details for the root user
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

    // @INFO hash the root user password to store in the database
    let mut hasher = Keccak256::new();
    hasher.update(root_password.as_bytes());
    let root_password_hash = hasher.finalize();
    let root_password_hash = format!("{:x}", root_password_hash);

    let time_stamp: DateTime<Utc> = SystemTime::now().into();
    let time_stamp = time_stamp.to_rfc3339();

    // @INFO create the root user
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

    // @INFO get the details for the user
    let id = Uuid::new_v4();
    let user_name = settings.get("server.network.user").unwrap_or_else(|| {
      warn!("No user name specified, using default username = user");
      "user".to_string()
    });
    let password = settings.get("server.network.password").unwrap_or_else(|| {
      warn!("No password specified, using default password = password");
      "password".to_string()
    });

    // @INFO hash the user password to store in the database
    let mut hasher = Keccak256::new();
    hasher.reset();
    hasher.update(password.as_bytes());
    let password_hash = hasher.finalize();
    let password_hash = format!("{:x}", password_hash);

    // @INFO create the user
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
