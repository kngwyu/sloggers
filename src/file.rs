//! File logger.
use std::fmt::Debug;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use slog::{Drain, FnValue, Logger};
use slog_async::Async;
use slog_term::{CompactFormat, FullFormat, PlainDecorator};

use {Build, Config, Result};
use misc::{module_and_line, timezone_to_timestamp_fn};
use types::{Format, Severity, TimeZone};

/// A logger builder which build loggers that write log records to the specified file.
///
/// The resulting logger will work asynchronously (the default channel size is 1024).
#[derive(Debug)]
pub struct FileLoggerBuilder {
    format: Format,
    timezone: TimeZone,
    level: Severity,
    appender: FileAppender,
    channel_size: usize,
}
impl FileLoggerBuilder {
    /// Makes a new `FileLoggerBuilder` instance.
    ///
    /// This builder will create a logger which uses `path` as
    /// the output destination of the log records.
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        FileLoggerBuilder {
            format: Format::default(),
            timezone: TimeZone::default(),
            level: Severity::default(),
            appender: FileAppender::new(path),
            channel_size: 1024,
        }
    }

    /// Sets the format of log records.
    pub fn format(&mut self, format: Format) -> &mut Self {
        self.format = format;
        self
    }

    /// Sets the time zone which this logger will use.
    pub fn timezone(&mut self, timezone: TimeZone) -> &mut Self {
        self.timezone = timezone;
        self
    }

    /// Sets the log level of this logger.
    pub fn level(&mut self, severity: Severity) -> &mut Self {
        self.level = severity;
        self
    }

    /// Sets the size of the asynchronous channel of this logger.
    pub fn channel_size(&mut self, channel_size: usize) -> &mut Self {
        self.channel_size = channel_size;
        self
    }

    fn build_with_drain<D>(&self, drain: D) -> Logger
    where
        D: Drain + Send + 'static,
        D::Err: Debug,
    {
        let drain = Async::new(drain.fuse())
            .chan_size(self.channel_size)
            .build()
            .fuse();
        let drain = self.level.set_level_filter(drain).fuse();
        Logger::root(drain, o!("module" => FnValue(module_and_line)))
    }
}
impl Build for FileLoggerBuilder {
    fn build(&self) -> Result<Logger> {
        let decorator = PlainDecorator::new(self.appender.clone());
        let timestamp = timezone_to_timestamp_fn(self.timezone);
        let logger = match self.format {
            Format::Full => {
                let format = FullFormat::new(decorator).use_custom_timestamp(timestamp);
                self.build_with_drain(format.build())
            }
            Format::Compact => {
                let format = CompactFormat::new(decorator).use_custom_timestamp(timestamp);
                self.build_with_drain(format.build())
            }
        };
        Ok(logger)
    }
}

#[derive(Debug)]
struct FileAppender {
    path: PathBuf,
    file: Option<File>,
}
impl Clone for FileAppender {
    fn clone(&self) -> Self {
        FileAppender {
            path: self.path.clone(),
            file: None,
        }
    }
}
impl FileAppender {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        FileAppender {
            path: path.as_ref().to_path_buf(),
            file: None,
        }
    }
    fn reopen_if_needed(&mut self) -> io::Result<()> {
        if !self.path.exists() || self.file.is_none() {
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .write(true)
                .open(&self.path)?;
            self.file = Some(file);
        }
        Ok(())
    }
}
impl Write for FileAppender {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.reopen_if_needed()?;
        if let Some(ref mut f) = self.file {
            f.write(buf)
        } else {
            Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Cannot open file: {:?}", self.path),
            ))
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        if let Some(ref mut f) = self.file {
            f.flush()?;
        }
        Ok(())
    }
}

/// The configuration of `FileLoggerBuilder`.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct FileLoggerConfig {
    /// Log level.
    #[serde(default)]
    pub level: Severity,

    /// Log record format.
    #[serde(default)]
    pub format: Format,

    /// Time Zone.
    #[serde(default)]
    pub timezone: TimeZone,

    /// Log file path.
    pub path: PathBuf,

    /// Asynchronous channel size
    #[serde(default = "default_channel_size")]
    pub channel_size: usize,
}
impl Config for FileLoggerConfig {
    type Builder = FileLoggerBuilder;
    fn try_to_builder(&self) -> Result<Self::Builder> {
        let mut builder = FileLoggerBuilder::new(&self.path);
        builder.level(self.level);
        builder.format(self.format);
        builder.timezone(self.timezone);
        builder.channel_size(self.channel_size);
        Ok(builder)
    }
}

fn default_channel_size() -> usize {
    1024
}
