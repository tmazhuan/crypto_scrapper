pub mod config;

// use config::*;
use crypto_scrapper::CoinMarketCapScrapper;

fn main() {
    let scrapper = CoinMarketCapScrapper::new(String::from("./config/config.toml"));
    let details = scrapper.get_details("iota");
    match details {
        Ok(r) => println!("{}", r),
        Err(e) => println!("{}", e),
    }
    println!("Price is: {}", scrapper.get_price(&String::from("iota")));
}

#[allow(unused)]
fn quick_test() {}
