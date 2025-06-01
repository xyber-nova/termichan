use termichan_config::{load_or_create_config, Config};
use std::sync::OnceLock;

pub static CONFIG: OnceLock<Config> = OnceLock::new();

fn main() {
    env_logger::init();

    let config = load_or_create_config(None).expect("Failed to load config.");
    CONFIG.set(config).expect("CONFIG has already initialized.");

    println!("{:#?}", CONFIG);
}
