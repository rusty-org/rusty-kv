//! Parser for the RESP (Redis Serialization Protocol).
//!
//! Provides functionality to parse RESP-formatted byte streams into Value objects.

use anyhow::Result;
use bytes::BytesMut;

use super::value::Value;

/// Parser for RESP-formatted data.
pub struct RespParser;

impl RespParser {
  /// Creates a new RESP parser.
  pub fn new() -> Self {
    Self
  }

  /// Parses RESP data from a buffer.
  ///
  /// # Arguments
  ///
  /// * `buf` - Buffer containing RESP-formatted data
  ///
  /// # Returns
  ///
  /// * `Ok(Some((Value, usize)))` - Parsed value and number of bytes consumed
  /// * `Ok(None)` - Not enough data to parse a complete value
  /// * `Err(...)` - Error during parsing
  pub fn parse_message(buf: &mut BytesMut) -> Result<Option<(Value, usize)>> {
    if buf.is_empty() {
      return Ok(None);
    }

    let parser = Self::new();

    // Parse based on the first byte (RESP type indicator)
    match buf[0] as char {
      '+' => parser.parse_simple_string(buf),
      '-' => parser.parse_error(buf),
      ':' => parser.parse_integer(buf),
      '$' => parser.parse_bulk_string(buf),
      '*' => parser.parse_array(buf),
      '#' => parser.parse_boolean(buf),
      _ => Err(anyhow::anyhow!(
        "Unknown RESP type: {:?}",
        std::str::from_utf8(&buf[..]).ok()
      )),
    }
  }

  /// Parses a RESP simple string ("+...").
  fn parse_simple_string(&self, buf: &BytesMut) -> Result<Option<(Value, usize)>> {
    self
      .parse_line(buf, 1)
      .map(|(line, len)| Some((Value::SimpleString(line), len)))
  }

  /// Parses a RESP error ("-...").
  fn parse_error(&self, buf: &BytesMut) -> Result<Option<(Value, usize)>> {
    self
      .parse_line(buf, 1)
      .map(|(line, len)| Some((Value::Error(line), len)))
  }

  /// Parses a RESP integer (":...").
  fn parse_integer(&self, buf: &BytesMut) -> Result<Option<(Value, usize)>> {
    self
      .parse_line(buf, 1)
      .and_then(|(line, len)| Ok(Some((Value::Integer(line.parse::<i64>()?), len))))
  }

  /// Parses a RESP bulk string ("$...").
  fn parse_bulk_string(&self, buf: &BytesMut) -> Result<Option<(Value, usize)>> {
    let (len_str, prefix_len) = self
      .read_until_crlf(&buf[1..])
      .ok_or_else(|| anyhow::anyhow!("Invalid bulk string length"))?;
    let len = self.parse_int(len_str)?;

    // Handle null strings ($-1\r\n)
    if len == -1 {
      return Ok(Some((Value::Null, 1 + prefix_len)));
    }

    let total_len = 1 + prefix_len + len as usize + 2;
    if buf.len() < total_len {
      return Ok(None);
    }
    let data = buf[1 + prefix_len..1 + prefix_len + len as usize].to_vec();
    let string = String::from_utf8(data)?;
    Ok(Some((Value::BulkString(string), total_len)))
  }

  /// Parses a RESP array ("*...").
  fn parse_array(&self, buf: &BytesMut) -> Result<Option<(Value, usize)>> {
    let (len_str, prefix_len) = self
      .read_until_crlf(&buf[1..])
      .ok_or_else(|| anyhow::anyhow!("Invalid array header"))?;
    let count = self.parse_int(len_str)?;

    // Handle null arrays (*-1\r\n)
    if count == -1 {
      return Ok(Some((Value::Null, 1 + prefix_len)));
    }

    let mut total_len = 1 + prefix_len;
    let mut values = Vec::new();

    // Parse each array element
    for _ in 0..count {
      let (v, len) = Self::parse_message(&mut BytesMut::from(&buf[total_len..]))?
        .ok_or_else(|| anyhow::anyhow!("Incomplete array element"))?;
      values.push(v);
      total_len += len;
    }

    Ok(Some((Value::Array(values), total_len)))
  }

  /// Parses a RESP boolean ("#...").
  fn parse_boolean(&self, buf: &BytesMut) -> Result<Option<(Value, usize)>> {
    if buf.len() < 4 {
      return Ok(None);
    }
    let val = match buf[1] as char {
      't' => true,
      'f' => false,
      _ => return Err(anyhow::anyhow!("Invalid boolean value")),
    };
    if &buf[2..4] != b"\r\n" {
      return Err(anyhow::anyhow!("Expected CRLF after boolean"));
    }
    Ok(Some((Value::Boolean(val), 4)))
  }

  /// Parses a line until CR-LF.
  fn parse_line(&self, buf: &BytesMut, start: usize) -> Result<(String, usize)> {
    self
      .read_until_crlf(&buf[start..])
      .ok_or_else(|| anyhow::anyhow!("CRLF not found"))
      .and_then(|(line, len)| Ok((String::from_utf8(line.to_vec())?, start + len)))
  }

  /// Parses a string as an integer.
  fn parse_int(&self, buf: &[u8]) -> Result<i64> {
    Ok(String::from_utf8(buf.to_vec())?.parse::<i64>()?)
  }

  /// Reads from a buffer until CR-LF is found.
  ///
  /// # Returns
  ///
  /// * `Some((&[u8], usize))` - Content before CR-LF and total length including CR-LF
  /// * `None` - CR-LF not found
  fn read_until_crlf<'a>(&self, buffer: &'a [u8]) -> Option<(&'a [u8], usize)> {
    for i in 1..buffer.len() {
      if buffer[i - 1] == b'\r' && buffer[i] == b'\n' {
        return Some((&buffer[0..i - 1], i + 1));
      }
    }
    None
  }
}
