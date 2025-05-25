//! Authentication command implementation.
//!
//! Handles user authentication against a database of credentials,
//! using secure password hashing (Keccak256).

use anyhow::{Result, anyhow};
use log::{info, warn};
use sha3::{Digest, Keccak256};

use crate::{
  resp::value::Value,
  storage::{db::InternalDB, memory::MemoryStore, memory::Store},
};

/// Authentication command handler.
///
/// Validates user credentials against the database and establishes
/// an authenticated session if successful.
pub struct AuthCommand;

impl AuthCommand {
  /// Executes the AUTH command.
  ///
  /// # Arguments
  ///
  /// * `args` - Command arguments (should contain username and password)
  /// * `store` - Memory store to set authentication state on
  /// * `db` - Database connection for credential verification
  ///
  /// # Returns
  ///
  /// * `Ok(Value)` - Authentication successful response
  /// * `Err` - Error if authentication fails or arguments are invalid
  ///
  /// # Example
  ///
  /// ```
  /// // Client sends: AUTH username password
  /// let result = AuthCommand::execute(vec!["username".to_string(), "password".to_string()], store, db).await;
  /// ```
  pub async fn execute(args: Vec<String>, store: MemoryStore, db: InternalDB) -> Result<Value> {
    if args.len() < 2 {
      return Err(anyhow!("AUTH requires username and password"));
    }

    let username = &args[0];
    let password = &args[1];

    // Hash the password for comparison
    let mut hasher = Keccak256::new();
    hasher.update(password.as_bytes());
    let password_hash = format!("{:x}", hasher.finalize());

    // Get a database connection from the pool
    let conn = db.pool.get()?;

    // Query the database for the user
    let mut stmt = conn.prepare("SELECT username, password FROM users WHERE username = ?")?;
    let mut rows = stmt.query(&[username])?;

    if let Some(row) = rows.next()? {
      let db_password: String = row.get(1)?;

      if db_password == password_hash {
        info!("User '{}' authenticated successfully", username);

        // Create a user-specific credential hash
        let mut hasher = Keccak256::new();
        hasher.update(format!("{}:{}", username, db_password).as_bytes());
        let credential_hash = format!("{:x}", hasher.finalize());

        // Set the current user in the store
        store.set_current_user(Some(credential_hash));

        return Ok(Value::SimpleString("OK".to_string()));
      } else {
        warn!("Invalid password for user '{}'", username);
        return Err(anyhow!("Invalid username or password"));
      }
    } else {
      warn!("User '{}' not found", username);
      return Err(anyhow!("Invalid username or password"));
    }
  }
}
