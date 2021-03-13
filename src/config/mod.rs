use serde::{Deserialize, Serialize};
use toml;

///Structure to store replacement expression which are used to replace sections inside a html text
#[derive(Serialize, Deserialize)]
pub struct Replace {
    pub from: String,
    pub to: String,
}
///Structure which holds the configuration details
#[derive(Serialize, Deserialize)]
pub struct Config {
    pub symbols: Vec<String>,
    pub regex_expressions: Vec<String>,
    pub replace_expressions: Vec<String>,
    pub about_regex: String,
    pub what_is_regex: String,
    pub title_regex: String,
    pub price_regex: String,
    pub replace: Vec<Replace>,
}
//The Configuration instance containing configuratio details and file location
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
impl Drop for ConfigObject {
    fn drop(&mut self) {
        self.store().unwrap();
    }
}

impl ConfigObject {
    /// Constructs a new `ConfigObject` with the specified configuration file `config_file`.
    pub fn new(config_file: String) -> ConfigObject {
        let config_text = std::fs::read_to_string(&config_file).unwrap();
        let config_const_values: Config = toml::from_str(&config_text).unwrap();
        ConfigObject {
            configuration: config_const_values,
            source: config_file,
        }
    }
    ///Deletes the symbol at index `i` from the `ConfigObject`
    pub fn delete_symbol(&mut self, i: usize) -> String {
        self.configuration.symbols.remove(i)
    }
    ///Adds the `symbol` to the `ConfigObject`
    pub fn add_symbol(&mut self, symbol: String) {
        &self.configuration.symbols.push(symbol);
    }
    ///Return the symbols stored in the `ConfigObject` inside of a `Vec`
    pub fn get_symbols(&self) -> Vec<String> {
        let mut result = Vec::new();
        for s in &self.configuration.symbols {
            result.push(String::from(s));
        }
        return result;
    }
    ///Stores the `ConfigObject` back to its file
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
