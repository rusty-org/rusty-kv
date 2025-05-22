//! Main entry point for the rusty-kv-server.
//!
//! This server implements a Redis-like key-value store using the RESP protocol.
//! It supports basic Redis commands and authentication, with data stored in memory
//! and user credentials persisted in SQLite.

// External dependencies
use log::{error, info, warn};
use tokio::net::TcpListener;

// Local dependencies
mod commands;
mod ds;
mod resp;
mod storage;
mod utils;

use storage::db::InternalDB;
use storage::memory::{MemoryStore, Store};
use utils::{logger::Logger, network::NetworkUtils, settings::Settings};

/// Main entry point function.
#[tokio::main(flavor = "multi_thread")]
async fn main() {
  // Set up logging
  Logger::setup();

  info!("Initializing RustyKV server...");

  // Load configuration
  let settings = Settings::new(Some("config.toml"));
  info!("Loaded settings from config.toml");

  warn!("Starting RustyKV server...");

  // Initialize the global memory store
  let memory_store = MemoryStore::new();
  info!("Initialized global memory store");

  // Initialize the internal database for persistence
  warn!("Initializing internal database...");
  let internal_db = InternalDB::new(&settings);

  // Get network configuration
  let kv_host = settings
    .get::<String>("server.network.host")
    .unwrap_or_else(|| {
      warn!("No host specified, using default");
      "127.0.0.1".to_string()
    });
  let kv_port = settings
    .get::<i16>("server.network.port")
    .unwrap_or_else(|| {
      warn!("No port specified, using default");
      6379
    });

  // Bind to the specified address and port
  let listener = TcpListener::bind(format!("{}:{}", kv_host, kv_port))
    .await
    .unwrap();

  warn!(
    "Bound to TCP - {:?}",
    listener.local_addr().unwrap_or_else(|e| {
      error!("Failed to get local address, {e}");
      std::net::SocketAddr::new("127.0.0.1".parse().unwrap(), 0)
    })
  );

  info!("Listening for incoming connections...");

  // Main server loop
  loop {
    let stream = listener.accept().await;
    match stream {
      Ok((stream, addr)) => {
        // Clone the store and db references for each connection
        let connection_store = memory_store.clone();
        let connection_db = internal_db.clone();

        // Spawn a new task to handle the connection
        tokio::spawn(async move {
          if let Err(e) =
            NetworkUtils::accept_connection(stream, connection_store, connection_db).await
          {
            error!("Error handling connection: {}", e);
          }
        });
        info!("Accepted a new connection from {}", addr);
      }
      Err(e) => {
        error!("Error accepting connection: {}", e);
      }
    }
  }
}
