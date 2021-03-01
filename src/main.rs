pub mod config;

use config::*;

fn main() {
    let config_file = Config::new(String::from("./config/config.toml"));
    println!("{}", config_file.about_regex);
}
