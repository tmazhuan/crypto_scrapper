// pub mod config {
use serde::{Deserialize, Serialize};
use toml;

#[derive(Serialize, Deserialize, Debug)]
pub struct Replace {
    pub from: String,
    pub to: String,
}
#[derive(Deserialize)]
pub struct Config {
    pub symbols: Vec<String>,
    pub regex_expressions: Vec<String>,
    pub replace_expressions: Vec<String>,
    pub about_regex: String,
    pub what_is_regex: String,
    pub replace: Vec<Replace>,
}

impl Default for Config {
    fn default() -> Self {
        //default config file is stored in ./config/config.toml
        let config_default_location = String::from("./config/config.toml");
        let config_text = std::fs::read_to_string(&config_default_location).unwrap();
        let config_const_values: Config = toml::from_str(&config_text).unwrap();
        config_const_values
    }
}

impl Config {
    pub fn new(config_file_location: String) -> Config {
        let config_text = std::fs::read_to_string(&config_file_location).unwrap();
        let config_const_values: Config = toml::from_str(&config_text).unwrap();
        config_const_values
    }
}
// }

#[cfg(test)]
mod tests {
    use super::Config;
    #[test]
    fn test_new() {
        let config_file = Config::new(String::from("./config/test.toml"));
        assert_eq!(config_file.about_regex, String::from("about_regex"));
        assert_eq!(config_file.what_is_regex, String::from("what_is_regex"));
        assert_eq!(config_file.symbols.len(), 3);
    }
    #[test]
    fn test_default() {
        let config_file: Config = Default::default();
        assert_eq!(
            config_file.about_regex,
            String::from(r#"<(h2)\s{1}.{1,20}\s{1}(class=".*?")>About.{1,30}</h2>"#)
        );
        assert_eq!(
            config_file.what_is_regex,
            String::from(r#"<(h\d)\s{1}(.{1,20}="what-is-.*?")>.*?</h\d>"#)
        );
        assert_eq!(config_file.symbols.len(), 3);
    }
}
