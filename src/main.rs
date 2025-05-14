// @NOTE External dependencies
use log::{error, info, warn};
use tokio::net::TcpListener;

// @NOTE Local dependencies
mod commands;
mod ds;
mod resp;
mod storage;
mod utils;

use storage::db::InternalDB;
use storage::memory::{MemoryStore, Store};
use utils::{logger::Logger, network::NetworkUtils, settings::Settings};

#[tokio::main(flavor = "multi_thread")]
async fn main() {
  Logger::setup();

  info!("Initializing Redis clone server...");

  let settings = Settings::new(Some("config.toml"));
  info!("Loaded settings from config.toml");

  warn!("Starting Redis clone server...");

  // Initialize the global memory store
  let memory_store = MemoryStore::new();
  info!("Initialized global memory store");

  warn!("Initializing internal database...");
  let internal_db = InternalDB::new(&settings);

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

  loop {
    let stream = listener.accept().await;
    match stream {
      Ok((stream, addr)) => {
        // Clone the store and db references for each connection
        let connection_store = memory_store.clone();
        let connection_db = internal_db.clone();

        tokio::spawn(async move {
          if let Err(e) = NetworkUtils::accept_connection(stream, connection_store, connection_db).await {
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
