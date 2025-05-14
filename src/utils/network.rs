//! Network handling utilities for TCP connections.
//!
//! Provides functionality for accepting and handling TCP connections,
//! processing RESP protocol commands, and routing them to the appropriate handlers.

use crate::{
  commands::executor::CommandExecutor,
  resp::{handler::RespHandler, value::Value},
  storage::{db::InternalDB, memory::MemoryStore},
};

use anyhow::Result;
use log::{debug, error, info};
use tokio::net::TcpStream;

/// Utilities for handling network operations.
pub struct NetworkUtils;

impl NetworkUtils {
  /// Handles a TCP connection by processing RESP commands.
  ///
  /// This function processes incoming RESP protocol commands from a TCP stream,
  /// executes them using the command executor, and sends back responses.
  ///
  /// # Arguments
  ///
  /// * `stream` - The TCP stream to read from and write to
  /// * `store` - The memory store for data storage and retrieval
  /// * `db` - The internal database for persisting data
  ///
  /// # Returns
  ///
  /// * `Ok(())` - Connection was handled successfully
  /// * `Err(...)` - An error occurred during connection handling
  pub async fn accept_connection(stream: TcpStream, store: MemoryStore, db: InternalDB) -> Result<()> {
    let peer_addr = stream.peer_addr()?;
    info!("Handling connection from: {}", peer_addr);

    debug!("Initializing RESP handler");
    let mut handler = RespHandler::new(stream);

    debug!("Initializing executor for incoming commands");
    let executor = CommandExecutor::new(store, db);

    // Main command processing loop
    while let Some(value) = handler.read_value().await? {
      debug!("Received: {:?}", value);

      if let Some((cmd, args)) = value.to_command() {
        info!("Command: {} with args: {:?}", cmd, args);

        // Execute the command and handle the result
        let result = executor.execute(&cmd, args).await;
        match result {
          Ok(response) => {
            handler.write_value(response).await?;
          }
          Err(e) => {
            let error_msg = format!("ERR {}", e);
            handler.write_value(Value::Error(error_msg)).await?;
          }
        }
      } else {
        error!("Error handling command, invalid format - {:?}", value);
        handler
          .write_value(Value::Error("ERR invalid command format".to_string()))
          .await?;
      }
    }

    info!("Connection closed: {}", peer_addr);
    Ok(())
  }
}
