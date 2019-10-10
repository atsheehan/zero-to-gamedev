use simplelog::{Config, LevelFilter, SimpleLogger};

const DEFAULT_LOG_LEVEL: LevelFilter = LevelFilter::Debug;

pub fn init_logging() {
    SimpleLogger::init(DEFAULT_LOG_LEVEL, Config::default()).unwrap();
}
