use crate::{
  commands::executor::CommandExecutor,
  resp::{handler::RespHandler, value::Value},
  storage::memory::MemoryStore,
  storage::memory::Store,
};

use anyhow::Result;
use log::{debug, error, info, warn};
use tokio::net::TcpStream;

pub struct NetworkUtils;

impl NetworkUtils {
  pub async fn accept_connection(stream: TcpStream) -> Result<()> {
    let peer_addr = stream.peer_addr()?;
    info!("Handling connection from: {}", peer_addr);

    debug!("Initializing RESP handler");
    let mut handler = RespHandler::new(stream);

    warn!("Initializing memory store");
    let memory_store = MemoryStore::new();

    debug!("Initializing executor for incoming commands");
    let executor = CommandExecutor::new(memory_store);

    while let Some(value) = handler.read_value().await? {
      debug!("Received: {:?}", value);

      if let Some((cmd, args)) = value.to_command() {
        info!("Command: {} with args: {:?}", cmd, args);

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
        error!("Eror handling command, invalid format - {:?}", value);
        handler
          .write_value(Value::Error("ERR invalid command format".to_string()))
          .await?;
      }
    }

    info!("Connection closed: {}", peer_addr);
    Ok(())
  }
}
