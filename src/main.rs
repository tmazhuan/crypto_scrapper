pub mod config;
pub mod frontend;

use crypto_scrapper::CoinMarketCapScrapper;
use frontend::*;

fn main() {
    let scrapper = match CoinMarketCapScrapper::new(String::from("./config/config.toml")) {
        Ok(s) => s,
        Err(e) => {
            println!("{}", e.to_string());
            return;
        }
    };
    cli_menu(scrapper);
}

#[allow(unused)]
fn quick_test() {}
