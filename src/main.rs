pub mod config;

// use config::*;
use crypto_scrapper::CoinMarketCapScrapper;
use regex::Regex;

fn main() {
    // quick_test();
    let scrapper = CoinMarketCapScrapper::new(String::from("./config/config.toml"));
    let details = scrapper.new_get_details("bitcoin");
    match details {
        Ok(r) => println!("{}", r),
        Err(e) => println!("{}", e),
    }
    // let price = scrapper.get_price(&String::from("cardano"));
    // match price {
    //     Ok((p, c)) => println!("Price is: {}\nPercentage Change is {}", p, c),
    //     Err(s) => println!("{}", s),
    // }
}

#[allow(unused)]
fn quick_test() {
    let source = r#"<h3 id="how-is-the-iota-network-secured">How Is the IOTA Network Secured?</h3>Given how the IOTA network isn’t a blockchain, you may not think that it would have much need for a consensus mechanism. However, to help keep the network secure, a relatively straightforward Proof-of-Work puzzle is included in the process of validating a transaction. There have been security concerns surrounding IOTA. In the past, researchers have claimed they have found vulnerabilities in the project’s code.<h3 id="where-can-you-buy-iota-miota">Where Can You Buy IOTA (MIOTA)?</h3>MIOTA is available on multiple exchanges — including Binance, Bitfinex, and OKEx. According to the project, a range of trading pairs are available, linking the token with Bitcoin, Ethereum, stablecoins, and fiat currencies including the Japanese yen, euro, pound, and dollar. Learn more about fiat on-ramps here."#;
    // let regex = r#"<h(?P<p>[1-9{1}])>\\s{1}id="{1,80}>(?P<t>.{1,80})</h[1-9]{1}>"#;
    let regex = r#"<h\d{1}.*?>(.*?)</h\d{1}>"#;
    let regex = Regex::new(&regex).unwrap();
    let result = regex
        .replace_all(&source, "\n------------$1------------\n")
        .into_owned();
    println!("{}", result);
}
