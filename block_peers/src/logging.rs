use simplelog::{Config, LevelFilter, SimpleLogger};

const DEFAULT_LOG_LEVEL: LevelFilter = LevelFilter::Trace;

pub fn init() {
    SimpleLogger::init(DEFAULT_LOG_LEVEL, Config::default()).unwrap();
}
