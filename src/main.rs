pub mod config;

// use config::*;
use crypto_scrapper::CoinMarketCapScrapper;
use std::time::Instant;
fn main() {
    // quick_test();
    let scrapper = CoinMarketCapScrapper::new(String::from("./config/config.toml"));
    // println!("in main after CoinMarketScrapper::new");
    // let symbols = vec![
    //     "iota".to_string(),
    //     "bitcoin".to_string(),
    //     "ethereum".to_string(),
    // ];
    // let price = scrapper.get_prices(&symbols);
    // match price {
    //     Ok(res) => {
    //         for r in res {
    //             println!("{}", r)
    //         }
    //     }
    //     Err(s) => println!("{}", s),
    // }
    let now = Instant::now();
    let price = scrapper.get_all_prices();
    let elapsed = now.elapsed().as_secs();
    match price {
        Ok(res) => {
            for r in res {
                println!("{}", r)
            }
        }
        Err(s) => println!("{}", s),
    }
    println!("\n\nParallel time elapsed: {}\n\n", elapsed);
    let symbols = scrapper.cfg.get_symbols();
    let mut result = Vec::new();
    let now = Instant::now();
    for s in symbols {
        result.push(scrapper.get_price(&s).unwrap());
    }
    let elapsed = now.elapsed().as_secs();
    for r in result {
        println!("{}", r)
    }
    println!("\n\nsequentiell time elapsed {}\n\n", elapsed);
    // println!("Result has {} entries", result.len());
    // let response = scrapper.get_market_data("iota", 3);
    // match response {
    //     Ok(res) => {
    //         for r in res {
    //             println!("{}", r);
    //         }
    //     }
    //     Err(e) => println!("{}", e),
    // }
    // let details = scrapper.get_details("bitcoin");
    // match details {
    //     Ok(r) => println!("{}", r),
    //     Err(e) => println!("{}", e),
    // }
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
