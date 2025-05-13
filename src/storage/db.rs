use std::{io::ErrorKind, sync::Arc};

use log::{error, warn};
use r2d2_sqlite::SqliteConnectionManager;

use crate::utils::settings::Settings;

#[derive(Clone, Debug)]
pub struct InternalDB {
  path: String,
  backup_path: String,
  backup_interval: u64,
  pool: Arc<r2d2::Pool<SqliteConnectionManager>>,
}

impl InternalDB {
  pub fn new(settings: Settings) -> Self {
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
          warn!("File already exists (harmless): {}", path);
        } else {
          error!("Failed to create file '{}': {}", path, e);
        }
      }
    }
  }

  fn create_dir(path: &str) {
    match std::fs::create_dir_all(path) {
      Ok(_) => warn!("Already existed: {}", path),
      Err(e) => error!("Failed to create '{}': {}", path, e),
    }
  }

  fn create_user(&self) {
    todo!()
  }
}
