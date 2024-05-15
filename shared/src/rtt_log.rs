use log::{LevelFilter, Metadata, Record};

pub struct Logger {
    level_filter: LevelFilter,
}

impl Logger {
    /// Static-friendly const initializer.
    ///
    /// * `level_filter`: The default level to enable.
    pub const fn new(level_filter: LevelFilter) -> Logger {
        Logger { level_filter }
    }
}

impl log::Log for Logger {
    /// Returns if logger is enabled.
    fn enabled(&self, metadata: &Metadata) -> bool {
        self.level_filter.ge(&metadata.level())
    }

    /// Log the record.
    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            rtt_target::rprintln!(
                "{:<5} [{}] {}",
                record.level(),
                record.target(),
                record.args()
            );
        }
    }

    fn flush(&self) {}
}
