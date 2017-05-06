use slog::Logger;

use {Result, Build, LoggerBuilder};
use file::FileLoggerConfig;
use null::NullLoggerConfig;
use terminal::TerminalLoggerConfig;

/// Configuration of a logger builder.
pub trait Config {
    /// Logger builder.
    type Builder: Build;

    /// Makes a logger builder associated with this configuration.
    fn try_to_builder(&self) -> Result<Self::Builder>;

    /// Builds a logger with this configuration.
    fn build_logger(&self) -> Result<Logger> {
        let builder = track_try!(self.try_to_builder());
        let logger = track_try!(builder.build());
        Ok(logger)
    }
}

/// The configuration of `LoggerBuilder`.
///
/// # Examples
///
/// Null logger.
///
/// ```
/// extern crate sloggers;
/// extern crate serdeconv;
///
/// use sloggers::{Config, LoggerConfig};
///
/// # fn main() {
/// let toml = r#"
/// type = "null"
/// "#;
/// let _config: LoggerConfig = serdeconv::from_toml_str(toml).unwrap();
/// # }
/// ```
///
/// Terminal logger.
///
/// ```
/// extern crate sloggers;
/// extern crate serdeconv;
///
/// use sloggers::{Config, LoggerConfig};
///
/// # fn main() {
/// let toml = r#"
/// type = "terminal"
/// level = "warning"
/// "#;
/// let _config: LoggerConfig = serdeconv::from_toml_str(toml).unwrap();
/// # }
/// ```
///
/// File logger.
///
/// ```
/// extern crate sloggers;
/// extern crate serdeconv;
///
/// use sloggers::{Config, LoggerConfig};
///
/// # fn main() {
/// let toml = r#"
/// type = "file"
/// path = "/path/to/file.log"
/// timezone = "utc"
/// "#;
/// let _config: LoggerConfig = serdeconv::from_toml_str(toml).unwrap();
/// # }
/// ```
#[allow(missing_docs)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum LoggerConfig {
    #[serde(rename = "file")]
    File(FileLoggerConfig),

    #[serde(rename = "null")]
    Null(NullLoggerConfig),

    #[serde(rename = "terminal")]
    Terminal(TerminalLoggerConfig),
}
impl Config for LoggerConfig {
    type Builder = LoggerBuilder;
    fn try_to_builder(&self) -> Result<Self::Builder> {
        match *self {
            LoggerConfig::File(ref c) => track!(c.try_to_builder()).map(LoggerBuilder::File),
            LoggerConfig::Null(ref c) => track!(c.try_to_builder()).map(LoggerBuilder::Null),
            LoggerConfig::Terminal(ref c) => {
                track!(c.try_to_builder()).map(LoggerBuilder::Terminal)
            }
        }
    }
}
