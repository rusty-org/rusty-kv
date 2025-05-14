//! TCP stream handler for RESP protocol.
//!
//! Provides functionality to read and write RESP values from/to a TCP stream.

use crate::resp::value::Value;
use anyhow::Result;
use bytes::{Buf, BytesMut};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use super::parser::RespParser;

/// Handles reading and writing RESP values from/to a TCP stream.
pub struct RespHandler {
  /// The TCP stream to read from and write to
  stream: TcpStream,
  /// Buffer for incoming data
  buffer: BytesMut,
}

impl RespHandler {
  /// Creates a new RESP handler for a TCP stream.
  ///
  /// # Arguments
  ///
  /// * `stream` - The TCP stream to handle
  pub fn new(stream: TcpStream) -> Self {
    Self {
      stream,
      buffer: BytesMut::with_capacity(1024),
    }
  }

  /// Reads a RESP value from the stream.
  ///
  /// # Returns
  ///
  /// * `Ok(Some(Value))` - Successfully read a value
  /// * `Ok(None)` - Connection closed with no data
  /// * `Err(...)` - Error reading or parsing data
  pub async fn read_value(&mut self) -> Result<Option<Value>> {
    loop {
      // Read data into the buffer
      let bytes_read = self.stream.read_buf(&mut self.buffer).await?;
      if bytes_read == 0 {
        if self.buffer.is_empty() {
          return Ok(None);
        } else {
          return Err(anyhow::anyhow!("Connection closed unexpectedly"));
        }
      }

      // Try to parse a RESP message from the buffer
      match RespParser::parse_message(&mut self.buffer) {
        Ok(Some((val, consumed))) => {
          self.buffer.advance(consumed);
          return Ok(Some(val));
        }
        Ok(None) => continue, // Not enough data, read more
        Err(e) => return Err(e),
      }
    }
  }

  /// Writes a RESP value to the stream.
  ///
  /// # Arguments
  ///
  /// * `value` - The value to write
  ///
  /// # Returns
  ///
  /// * `Ok(())` - Value was successfully written
  /// * `Err(...)` - Error writing to the stream
  pub async fn write_value(&mut self, value: Value) -> Result<()> {
    let data = value.serialize();
    self.stream.write_all(data.as_bytes()).await?;
    Ok(())
  }
}
