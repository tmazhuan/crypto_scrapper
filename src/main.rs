pub mod config;
pub mod frontend;

use crypto_scrapper::CoinMarketCapScrapper;
use frontend::*;

fn main() {
    // quick_test();
    let scrapper = match CoinMarketCapScrapper::new(String::from("./config/config.toml")) {
        Ok(s) => s,
        Err(e) => {
            println!("{}", e.to_string());
            return;
        }
    };
    cli_menu(scrapper);
    // let now = Instant::now();
    // let price = scrapper.get_all_prices();
    // let elapsed = now.elapsed().as_secs();
    // let mut clip_ = String::new();
    // match price {
    //     Ok(res) => {
    //         clip_ = res.iter().fold(String::new(), |clip, x| {
    //             format!("{}{}\n", clip, x.to_string())
    //         });
    //         // for r in res {
    //         //     clip_.push_str(&r.to_string());
    //         //     clip_.push('\n');
    //         // }
    //     }
    //     Err(s) => println!("{}", s),
    // }
    // to_clip(clip_);
    // println!("\n\nParallel time elapsed: {}\n\n", elapsed);
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
