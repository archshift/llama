extern crate lgl;
extern crate log;

use std::sync::atomic::{AtomicBool, Ordering};

struct UILogger;

static TRACE_ENABLED: AtomicBool = AtomicBool::new(false);

impl log::Log for UILogger {
    fn enabled(&self, _: &log::LogMetadata) -> bool {
        true
    }

    fn log(&self, record: &log::LogRecord) {
        if !self.enabled(record.metadata()) { return }

        if record.level() > log::LogLevel::Debug && !TRACE_ENABLED.load(Ordering::Relaxed) { return }

        let thread = ::std::thread::current();
        let thread_name: String = if let Some(name) = thread.name() {
            name.into()
        } else {
            format!("{:?}", thread.id())
        };
        let string = format!("[[{}]] ## {}: {}\n", thread_name, record.level(), record.args());
        print!("{}", string);
        lgl::log(&string);
    }
}

pub fn init() -> Result<(), log::SetLoggerError> {
    log::set_logger(|max| {
        max.set(log::LogLevelFilter::Trace);
        Box::new(UILogger)
    })
}

pub fn allow_trace(yes: bool) {
    TRACE_ENABLED.store(yes, Ordering::Relaxed);
}
