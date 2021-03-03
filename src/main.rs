pub mod config;

// use config::*;
use crypto_scrapper::CoinMarketCapScrapper;

fn main() {
    let scrapper = CoinMarketCapScrapper::new(String::from("./config/config.toml"));
    // let details = scrapper.get_details("iota");
    // match details {
    //     Ok(r) => println!("{}", r),
    //     Err(e) => println!("{}", e),
    // }
    let price = scrapper.get_price(&String::from("akie"));
    match price {
        Ok((p, c)) => println!("Price is: {}\nPercentage Change is {}", p, c),
        Err(s) => println!("{}", s),
    }
}

#[allow(unused)]
fn quick_test() {}
