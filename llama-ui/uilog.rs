extern crate lgl;
extern crate log;

struct UILogger;

impl log::Log for UILogger {
    fn enabled(&self, metadata: &log::LogMetadata) -> bool {
        true
    }

    fn log(&self, record: &log::LogRecord) {
        if self.enabled(record.metadata()) {
            let string = format!("{}: {}\n", record.level(), record.args());
            lgl::log(&string);
        }
    }
}

pub fn init() -> Result<(), log::SetLoggerError> {
    log::set_logger(|max| {
        max.set(log::LogLevelFilter::Trace);
        Box::new(UILogger)
    })
}