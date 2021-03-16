pub mod config;
pub mod frontend;

use crypto_scrapper::CoinMarketCapScrapper;
use frontend::*;

fn main() {
    let scrapper = match CoinMarketCapScrapper::new(String::from("./config/config.toml")) {
        Ok(s) => s,
        Err(e) => {
            println!(
                "{}\nMake sure that you specify an existing configuration file.",
                e.to_string()
            );
            return;
        }
    };
    cli_menu(scrapper);
}
