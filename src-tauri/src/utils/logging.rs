use log::LevelFilter;

pub fn init_logging() {
    env_logger::Builder::new()
        .filter_level(LevelFilter::Debug)
        .init();
}
