pub mod config;

// use config::*;
use crypto_scrapper::CoinMarketCapScrapper;

fn main() {
    // quick_test();
    let mut scrapper = CoinMarketCapScrapper::new(String::from("./config/config.toml"));
    println!("in main after CoinMarketScrapper::new");
    let response = scrapper.get_market_data("iota", 3);
    match response {
        Ok(res) => {
            for r in res {
                println!("{}", r);
            }
        }
        Err(e) => println!("{}", e),
    }
    let details = scrapper.get_details("bitcoin");
    match details {
        Ok(r) => println!("{}", r),
        Err(e) => println!("{}", e),
    }
    let price = scrapper.get_price(&String::from("cardano"));
    match price {
        Ok((p, c)) => println!("Price is: {}\nPercentage Change is {}", p, c),
        Err(s) => println!("{}", s),
    }
}

#[allow(unused)]
fn quick_test() {
    // let url = format!("https://coinmarketcap.com/currencies/{}/markets", "bitcoin");
    // println!("URL: {}", url);
    // // let html = reqwest::blocking::get(&url).unwrap().text().unwrap();
    // let mut file = File::create("bitcoin.html").unwrap();
    // file.write_all(html.as_bytes());

    // let regex = r#"<(table) (class=".*?currencies-markets_.*? ")>"#;
    // let position = html.find("Bitcoin Markets").unwrap();
    // println!("position {}", position);
    // let slice = &html[position - 500..position + 500];
    // println!("html around:\n{}", slice);

    // let regex = Regex::new(&regex).unwrap();
    // let result = regex.captures(&html);
    // println!("{:?}", result);
}
