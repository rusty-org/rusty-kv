//! Authentication command implementation.
//!
//! It returns the current user if authenticated, or an error if not.

use anyhow::{Ok, Result, anyhow};
use log::{debug, warn};
use rusqlite::params;
use sha3::{Digest, Keccak256};

use crate::{
  resp::value::Value,
  storage::{db::InternalDB, memory::MemoryStore, memory::Store},
};

/// WhoAmi command handler.
///
/// This command checks the current authenticated user and returns their username
/// and credential hash if they are authenticated.
/// If the user is not authenticated, it returns an error.
pub struct WhoAmi;
impl WhoAmi {
  /// This command returns the current username and its credential hash.
  /// It checks if the user is authenticated and retrieves their information
  /// from the database, returning it in a RESP-compatible format.
  ///
  /// # Example
  /// ```
  /// // Client sends: WHOAMI
  /// let result = WhoAmi::execute(store, db).await;
  /// ```
  pub async fn execute(store: MemoryStore, db: InternalDB) -> Result<Value> {
    // First check if the user is authenticated
    if !store.is_authenticated() {
      return Err(anyhow!("Not authenticated"));
    }

    // Get the current user's credential hash
    let current_hash = store.get_current_user().unwrap();
    debug!("Current user hash: {}", current_hash);

    // Get a database connection from the pool
    let conn = db.pool.get()?;

    // Query the database for all users
    let mut stmt = conn.prepare("SELECT username, password FROM users")?;
    let mut rows = stmt.query(params![])?;

    while let Some(row) = rows.next()? {
      let username: String = row.get(0)?;
      let password: String = row.get(1)?;
      debug!("Checking username: {}", username);

      // Create hash directly - avoid additional query
      let hash_input = format!("{}:{}", username, password);

      let mut hasher = Keccak256::new();
      hasher.update(hash_input.as_bytes());
      let recreated_hash = format!("{:x}", hasher.finalize());

      // Compare the recreated hash with our current hash
      if recreated_hash == current_hash {
        return Ok(Value::BulkString(format!(
          "Current user: {} ({})",
          username, current_hash
        )));
      }
    }

    // If we get here, we didn't find a matching user
    warn!("Could not find user matching the current credential hash");
    Err(anyhow!("User not found in database"))
  }
}
