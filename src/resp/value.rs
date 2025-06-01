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
  pub fn to_command(&self) -> Option<(String, Vec<Value>)> {
    if let Value::Array(elements) = self {
      if elements.is_empty() {
        return None;
      }

      // Extract the actual command from possibly embedded RESP format
      let command = match &elements[0] {
        Value::BulkString(s) => {
          // Check if the string is in RESP format ($3\r\nset\r\n)
          if s.starts_with('$') && s.contains("\r\n") {
            // Extract actual command from embedded RESP format
            let parts: Vec<&str> = s.split("\r\n").collect();
            if parts.len() >= 2 {
              parts[1].to_string().to_uppercase()
            } else {
              s.clone().to_uppercase()
            }
          } else {
            s.clone().to_uppercase()
          }
        }
        Value::SimpleString(s) => s.clone().to_uppercase(),
        _ => return None,
      };

      // Extract arguments, preserving their original types
      let args = elements[1..]
        .iter()
        .map(|v| match v {
          Value::BulkString(s) => {
            // Check if the string is in RESP format
            if s.starts_with('$') && s.contains("\r\n") {
              // Extract actual value from embedded RESP format
              let parts: Vec<&str> = s.split("\r\n").collect();
              if parts.len() >= 2 {
                Value::BulkString(parts[1].to_string())
              } else {
                v.clone()
              }
            } else if s.starts_with(':') {
              // Handle numeric values encoded as :100\r\n
              let num_str = s.trim_start_matches(':').trim_end_matches("\r\n");
              if let Ok(num) = num_str.parse::<i64>() {
                Value::Integer(num)
              } else {
                v.clone()
              }
            } else if s.starts_with("#") {
              // Handle boolean values encoded as #t\r\n or #f\r\n
              let bool_str = s.trim_start_matches('#').trim_end_matches("\r\n");
              if bool_str == "t" {
                Value::Boolean(true)
              } else if bool_str == "f" {
                Value::Boolean(false)
              } else {
                v.clone()
              }
            } else if s.starts_with("*") {
              // ----------------------------------------------------------------------
              // Handle array values encoded as *3\r\n$1\r\n1\r\n$1\r\n2\r\n$1\r\n3\r\n
              let mut lines = s.split_terminator("\r\n");

              // Extract the array header, e.g., "*3"
              let arr_header = lines.next().unwrap_or("");
              let arr_length: usize = if arr_header.starts_with('*') {
                arr_header[1..].parse().unwrap_or(0)
              } else {
                0
              };

              let mut result = Vec::<(usize, String)>::with_capacity(arr_length);

              while let Some(length_str) = lines.next() {
                if length_str.starts_with('$') {
                  if let Ok(length) = length_str[1..].parse::<usize>() {
                    if let Some(value) = lines.next() {
                      result.push((length, value.to_string()));
                    }
                  }
                }
              }

              // ----------------------------------------------------------------------

              let parts: Vec<&str> = s.split("\r\n").collect();
              if parts.len() >= 2 {
                let array_length = parts[0]
                  .trim_start_matches('*')
                  .parse::<usize>()
                  .unwrap_or(0);
                let mut array_values = Vec::new();
                for i in 1..=array_length {
                  if i < parts.len() {
                    array_values.push(Value::BulkString(parts[i].to_string()));
                  }
                }
                Value::Array(array_values)
              } else {
                v.clone()
              }
            } else {
              v.clone()
            }
          }
          _ => v.clone(),
        })
        .collect();

      Some((command, args))
    } else {
      None
    }
  }
}
