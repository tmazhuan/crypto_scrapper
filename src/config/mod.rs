use serde::{Deserialize, Serialize};
use toml;

#[derive(Serialize, Deserialize)]
pub struct Replace {
    pub from: String,
    pub to: String,
}
#[derive(Serialize, Deserialize)]
pub struct Config {
    pub symbols: Vec<String>,
    pub regex_expressions: Vec<String>,
    pub replace_expressions: Vec<String>,
    pub about_regex: String,
    pub what_is_regex: String,
    pub price_regex: String,
    pub replace: Vec<Replace>,
}
pub struct ConfigObject {
    pub configuration: Config,
    source: String,
}
impl Default for ConfigObject {
    fn default() -> Self {
        //default config file is stored in ./config/config.toml
        let config_default_location = String::from("./config/config.toml");
        let config_text = std::fs::read_to_string(&config_default_location).unwrap();
        let config_const_values: Config = toml::from_str(&config_text).unwrap();
        ConfigObject {
            configuration: config_const_values,
            source: config_default_location,
        }
    }
}

impl ConfigObject {
    pub fn new(config_file_location: String) -> ConfigObject {
        let config_text = std::fs::read_to_string(&config_file_location).unwrap();
        let config_const_values: Config = toml::from_str(&config_text).unwrap();
        ConfigObject {
            configuration: config_const_values,
            source: config_file_location,
        }
    }

    pub fn add_symbol(&mut self, symbol: String) {
        &self.configuration.symbols.push(symbol);
    }

    pub fn store(&self) -> std::io::Result<()> {
        std::fs::write(
            &self.source,
            toml::to_string(&self.configuration).unwrap().as_bytes(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::ConfigObject;
    #[test]
    fn test_new() {
        let config_file = ConfigObject::new(String::from("./config/test.toml"));
        assert_eq!(
            config_file.configuration.about_regex,
            String::from("about_regex")
        );
        assert_eq!(
            config_file.configuration.what_is_regex,
            String::from("what_is_regex")
        );
        assert_eq!(config_file.configuration.symbols.len(), 3);
    }
    #[test]
    fn test_default() {
        let config_file: ConfigObject = Default::default();
        assert_eq!(
            config_file.configuration.about_regex,
            String::from(r#"<(h2)\s{1}.{1,20}\s{1}(class=".*?")>About.{1,30}</h2>"#)
        );
        assert_eq!(
            config_file.configuration.what_is_regex,
            String::from(r#"<(h\d)\s{1}(.{1,20}="what-is-.*?")>.*?</h\d>"#)
        );
        assert_eq!(config_file.configuration.symbols.len(), 3);
    }
}
