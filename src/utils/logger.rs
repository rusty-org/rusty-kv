//! Logging utilities for the server.
//!
//! Provides a simple and standardized logging setup for the entire application.

use log::info;
use simple_logger::SimpleLogger;

/// Logging configuration utility.
pub struct Logger;

impl Logger {
  /// Sets up the default logger for the application.
  ///
  /// Configures a SimpleLogger with colored output, trace-level logging,
  /// and timestamps in ISO 8601 format.
  pub fn setup() {
    SimpleLogger::new()
      .with_colors(true)
      .with_level(log::LevelFilter::Trace)
      .with_timestamp_format(time::macros::format_description!(
        "[year]-[month]-[day] [hour]:[minute]:[second]"
      ))
      .init()
      .unwrap();
    info!("Setting up default logger !")
  }
}
