//! RESP (Redis Serialization Protocol) value types.
//!
//! Defines the different value types that can be serialized and deserialized
//! according to the RESP specification.

/// Enum representing the different RESP value types.
#[derive(Clone, Debug)]
pub enum Value {
  /// Null value (represented as "$-1\r\n" in RESP)
  Null,

  /// Simple string (represented as "+{string}\r\n" in RESP)
  SimpleString(String),

  /// Bulk string (represented as "${length}\r\n{string}\r\n" in RESP)
  BulkString(String),

  /// Array of values (represented as "*{length}\r\n{values...}" in RESP)
  Array(Vec<Value>),

  /// Error message (represented as "-{message}\r\n" in RESP)
  Error(String),

  /// Integer (represented as ":{integer}\r\n" in RESP)
  Integer(i64),

  /// Boolean (represented as "#{t|f}\r\n" in RESP)
  Boolean(bool),
}

impl Value {
  /// Serializes the value to a RESP-compatible string.
  ///
  /// # Returns
  ///
  /// A string containing the RESP-encoded representation of the value.
  pub fn serialize(&self) -> String {
    match self {
      Value::Null => "$-1\r\n".to_string(),
      Value::SimpleString(s) => format!("+{}\r\n", s),
      Value::BulkString(s) => format!("${}\r\n{}\r\n", s.len(), s),
      Value::Integer(i) => format!(":{}\r\n", i),
      Value::Error(s) => format!("-{}\r\n", s),
      Value::Boolean(b) => format!("#{}\r\n", if *b { "t" } else { "f" }),
      Value::Array(arr) => {
        let mut s = format!("*{}\r\n", arr.len());
        for v in arr {
          s.push_str(&v.serialize());
        }
        s
      }
    }
  }

  /// Converts a RESP value to a command and arguments.
  ///
  /// Expects an array where the first element is the command name
  /// and subsequent elements are arguments.
  ///
  /// # Returns
  ///
  /// * `Some((String, Vec<String>))` - Command name (uppercase) and argument list
  /// * `None` - If the value is not a valid command format
  pub fn to_command(&self) -> Option<(String, Vec<String>)> {
    if let Value::Array(elements) = self {
      if elements.is_empty() {
        return None;
      }
      let command = match &elements[0] {
        Value::BulkString(s) => s.clone(),
        Value::SimpleString(s) => s.clone(),
        _ => return None,
      };
      let args = elements[1..]
        .iter()
        .filter_map(|v| match v {
          Value::BulkString(s) | Value::SimpleString(s) => Some(s.clone()),
          _ => None,
        })
        .collect();
      Some((command.to_uppercase(), args))
    } else {
      None
    }
  }
}
