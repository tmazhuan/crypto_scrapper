pub mod config;

use config::*;

use serde::Serialize;

#[derive(Serialize)]
struct Config {
    ip: String,
    port: Option<u16>,
    keys: Keys,
}

#[derive(Serialize)]
struct Keys {
    github: String,
    travis: Option<String>,
}

fn main() {
    let mut config_object = ConfigObject::new(String::from("./config/config.toml"));
    println!("{}", config_object.configuration.about_regex);
    config_object.configuration.symbols = vec![
        String::from("abc"),
        String::from("def"),
        String::from("ghi"),
    ];
    match config_object.store() {
        Ok(_) => println!("Configuration Stored"),
        Err(e) => println!("Configuration not stored. Error: {}", e),
    }
    // quick_test();
}

#[allow(unused)]
fn quick_test() {
    let config = Config {
        ip: "33.33.33.33".to_string(),
        port: None,
        keys: Keys {
            github: "lkasjs".to_string(),
            travis: Some("sadfa3".to_string()),
        },
    };
    let config_default_location = String::from("./config/xxx.toml");
    let toml = toml::to_string(&config).unwrap();
    std::fs::write(config_default_location, toml.as_bytes()).unwrap();
}
