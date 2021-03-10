pub mod config;

use config::ConfigObject;
use html_parser::ElementRelation::Child;
use html_parser::{ElementRelation, HtmlParser, ParseError};
use regex::Regex;
use std::fmt;
use std::fmt::Display;

pub struct MarketResult {
    source: String,
    pair: String,
    price: f64,
    volume: f64,
    volume_percent: f64,
}

impl MarketResult {
    pub fn get_volume_in_dollars(&self) -> f64 {
        if self.price == 0.0 || self.volume == 0.0 {
            0.0
        } else {
            self.volume / self.price
        }
    }
    fn get_spaces(&self, i: usize) -> String {
        let mut temp = String::new();
        for _ in 0..i {
            temp.push_str(" ");
        }
        String::from(&temp)
    }
}
impl Display for MarketResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let column_widht = 15;
        let mut result = String::from(&self.source);
        // let mut len = self.source.len();
        // println!("{}", column_widht - len);
        result.push_str(&self.get_spaces(column_widht - self.source.len()));
        result.push_str(&self.pair);
        // len = self.pair.len();
        // println!("{}", column_widht - len);
        result.push_str(&self.get_spaces(column_widht - self.pair.len()));
        let price = format!("{}", (self.price));
        // len = price.len();
        // println!("{}", column_widht - len);
        result.push_str(&price);
        result.push_str(&self.get_spaces(column_widht - price.len()));
        let vol = format!("{}", (self.volume));
        // len = vol.len();
        // println!("{}", column_widht - len);
        result.push_str(&vol);
        result.push_str(&self.get_spaces(column_widht - vol.len()));
        let vol_per = format!("{}", (self.volume_percent));
        result.push_str(&vol_per);
        write!(f, "{}", result)
    }
}

pub struct CoinMarketCapScrapper {
    cfg: ConfigObject,
    html_parser: HtmlParser,
    runtime: tokio::runtime::Runtime,
}

impl Drop for CoinMarketCapScrapper {
    fn drop(&mut self) {
        self.runtime.block_on(self.html_parser.close_connection());
    }
}

impl CoinMarketCapScrapper {
    pub fn new(config_file_location: String) -> CoinMarketCapScrapper {
        println!("in CoinMarketCapScrapper::new");
        let rt = tokio::runtime::Runtime::new().unwrap();
        CoinMarketCapScrapper {
            cfg: ConfigObject::new(config_file_location),
            html_parser: rt.block_on(async { HtmlParser::new().await }),
            runtime: rt,
        }
    }
    pub fn get_details(&mut self, symbol: &str) -> Result<String, ParseError> {
        println!("in CoinMarketCapScrapper::new_get_details");

        let url = format!("https://coinmarketcap.com/currencies/{}/", symbol);
        // let html = reqwest::blocking::get(&url).unwrap().text().unwrap();
        let html = self
            .runtime
            .block_on(self.html_parser.get_html_source(&url, false))
            .unwrap();
        // let html = self.html_parser.get_html_source(&url, false)?;
        let mut what_is_inner = match self.html_parser.get_inner_html_from_element(
            &self.cfg.configuration.what_is_regex,
            &html,
            vec![vec![ElementRelation::Parent]],
        ) {
            Ok(s) => String::from(&s[0]),
            Err(_) => {
                match self.html_parser.get_inner_html_from_element(
                    &self.cfg.configuration.about_regex,
                    &html,
                    vec![vec![ElementRelation::Parent]],
                ) {
                    Ok(s) => String::from(&s[0]),
                    Err(e) => return Err(e),
                }
            }
        };
        what_is_inner = self.cleanup_result_string(what_is_inner);
        Ok(what_is_inner)
    }

    fn cleanup_number(&self, input: &String) -> String {
        let regex = r#"[^\d.]"#;
        let regex = Regex::new(&regex).unwrap();
        String::from(regex.replace_all(&input, ""))
    }
    fn cleanup_result_string(&self, mut text: String) -> String {
        for expr in &self.cfg.configuration.regex_expressions {
            let regex = Regex::new(&expr).unwrap();
            text = regex.replace_all(&text, "").into_owned();
        }
        for expr in &self.cfg.configuration.replace {
            let regex = Regex::new(&expr.from).unwrap();
            text = regex.replace_all(&text, expr.to.as_str()).into_owned();
        }
        text = text.replace("\\n", &format!("\n"));
        for expr in &self.cfg.configuration.replace_expressions {
            text = text.replace(expr, "");
        }
        let regex = Regex::new(&self.cfg.configuration.title_regex).unwrap();
        text = regex
            .replace_all(&text, "\n------------$1------------\n")
            .into_owned();
        String::from(text)
    }

    pub fn get_price(&mut self, symbol: &str) -> Result<(f64, f64), ParseError> {
        let url = format!("https://coinmarketcap.com/currencies/{}/", symbol);
        // let html = reqwest::blocking::get(&url).unwrap().text().unwrap();
        // let html = self.html_parser.get_html_source(&url, true)?;
        let html = self
            .runtime
            .block_on(self.html_parser.get_html_source(&url, false))
            .unwrap();
        let reg_price_section = r#"(div) (class=".{1,20}priceTitle__.{1,20}")>"#;
        //price
        let price = match self.html_parser.get_inner_html_from_element(
            reg_price_section,
            &html,
            vec![vec![ElementRelation::Child(0)]],
        ) {
            Ok(s) => String::from(&s[0]),
            Err(e) => return Err(e),
        };
        let price = price
            .replace("$", "")
            .replace(",", "")
            .parse::<f64>()
            .unwrap();
        //Percentage
        let percentage = match self.html_parser.get_inner_html_from_element(
            reg_price_section,
            &html,
            vec![vec![ElementRelation::Child(0), ElementRelation::Sibling(0)]],
        ) {
            Ok(s) => String::from(&s[0]),
            Err(e) => return Err(e),
        };
        let per_regex = r#"<span class="(.{1,20})"></span>([0-9]+[.]?[0-9]*)<!-- -->%"#;
        let re = Regex::new(per_regex).unwrap();
        let cap = re.captures(&percentage);
        let percentage = match cap {
            Some(c) => {
                let p = String::from(&c[2]).parse::<f64>().unwrap();
                let s = match String::from(&c[1]).find("icon-Caret-up") {
                    Some(_) => 1.0,
                    None => -1.0,
                };
                p * s
            }
            None => {
                return Err(ParseError::new(format!(
                    "Error when reading page. Check manually on {}",
                    url
                )));
            }
        };
        Ok((price, percentage))
    }

    #[allow(unused)]
    pub fn get_7d_performance(&self, symbol: &str) -> f64 {
        0.0
    }

    pub fn get_market_data(
        &mut self,
        symbol: &str,
        number_of_results: i32,
    ) -> Result<Vec<MarketResult>, ParseError> {
        let mut result = Vec::new();
        for i in 0..number_of_results {
            let url = format!("https://coinmarketcap.com/currencies/{}/markets", symbol);
            // let html = reqwest::blocking::get(&url).unwrap().text().unwrap();
            // let html = self.html_parser.get_html_source(&url, false)?;
            let html = self
                .runtime
                .block_on(self.html_parser.get_html_source(&url, false))
                .unwrap();
            let regex = r#"<(table) (class=".*?currencies-markets_.*? ")>"#;
            let rel_source = vec![vec![
                Child(1),
                Child(i),
                Child(1),
                Child(0),
                Child(0),
                Child(1),
                Child(0),
            ]];
            let rel_pairs = vec![vec![Child(1), Child(i), Child(2), Child(0), Child(0)]];
            let rel_price = vec![vec![Child(1), Child(i), Child(3)]];
            let rel_vol = vec![vec![Child(1), Child(i), Child(4), Child(0)]];
            let rel_vol_perc = vec![vec![Child(1), Child(1), Child(5), Child(0), Child(0)]];
            let inner_source = self
                .html_parser
                .get_inner_html_from_element(regex, &html, rel_source)?;
            let inner_pairs = self
                .html_parser
                .get_inner_html_from_element(regex, &html, rel_pairs)?;
            let inner_price = self
                .html_parser
                .get_inner_html_from_element(regex, &html, rel_price)?;
            let inner_vol = self
                .html_parser
                .get_inner_html_from_element(regex, &html, rel_vol)?;
            let inner_vol_perc =
                self.html_parser
                    .get_inner_html_from_element(regex, &html, rel_vol_perc)?;
            result.push(MarketResult {
                source: String::from(&inner_source[0]),
                pair: String::from(&inner_pairs[0]),
                price: self.cleanup_number(&inner_price[0]).parse::<f64>().unwrap(),
                volume: self.cleanup_number(&inner_vol[0]).parse::<f64>().unwrap(),
                volume_percent: self
                    .cleanup_number(&inner_vol_perc[0])
                    .parse::<f64>()
                    .unwrap(),
            });
        } //for
        return Ok(result);
    } //fn get_market_data
}

mod html_parser {
    use fantoccini::ClientBuilder;
    use regex::Regex;
    use scraper::{ElementRef, Html, Selector};
    use std::collections::HashMap;
    use std::error::Error;
    use std::fmt;
    // use tokio::runtime::Runtime;

    pub struct ParseError {
        details: String,
    }
    impl ParseError {
        pub fn new(details: String) -> ParseError {
            ParseError { details }
        }
    }
    impl fmt::Display for ParseError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}", self.details)
        }
    }
    impl fmt::Debug for ParseError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}", self.details)
        }
    }
    impl Error for ParseError {
        fn description(&self) -> &str {
            &self.details
        }
    }

    pub enum ElementRelation {
        Parent,
        Child(i32),
        Sibling(i32),
    }

    pub struct HtmlParser {
        client: fantoccini::Client,
        cache: HashMap<String, String>,
    }

    impl HtmlParser {
        pub async fn new() -> HtmlParser {
            return HtmlParser {
                client: ClientBuilder::native()
                    .connect("http://localhost:9515")
                    .await
                    .unwrap(),
                cache: HashMap::new(),
            };
        }

        pub async fn close_connection(&mut self) {
            self.client.close().await.unwrap();
        }

        pub async fn get_html_source(
            &mut self,
            url: &str,
            reload: bool,
        ) -> Result<String, ParseError> {
            //first lets check if we have the source already in the cache if we don't need to reload
            if !reload {
                //check if data is alread in the cache
                if self.cache.contains_key(url) {
                    return Ok(String::from(self.cache.get(url).unwrap()));
                }
            };
            // let rt = Runtime::new().unwrap();
            // let mut source = String::new();
            // rt.block_on(async {
            self.client.goto(url).await.unwrap();
            let source = self.client.source().await.unwrap();
            // let _re = self.client.close().await.unwrap();
            // });
            //add data to cache
            self.cache.insert(String::from(url), String::from(&source));
            Ok(source)
        }
        pub fn get_inner_html_from_element(
            &self,
            regex: &str,
            source: &str,
            relations: Vec<Vec<ElementRelation>>,
        ) -> Result<Vec<String>, ParseError> {
            let re = Regex::new(regex).unwrap();
            let cap = re.captures(source);
            let (tag, attribute) = match cap {
                Some(c) => (String::from(&c[1]), String::from(&c[2])),
                None => {
                    println!("Regex: {}", regex);
                    // println!("html: {}", source);
                    return Err(ParseError::new(String::from(
                        "Element not found. Please check manually",
                    )));
                }
            };
            // println!("tag: {}, attribute: {}", tag, attribute);
            let selector = Selector::parse(&format!("{}[{}]", tag, attribute)).unwrap();
            // let element = document.select(&selector).next().unwrap();
            let document = Html::parse_document(&source);
            let r = document.select(&selector).next().unwrap();
            let mut result: Vec<String> = Vec::new();
            for rel in relations {
                let inner = navigate_relation(rel, r);
                result.push(inner.inner_html())
            }
            return Ok(result);
        }
    }

    fn navigate_relation(rel: Vec<ElementRelation>, element: scraper::ElementRef) -> ElementRef {
        let mut result = element.clone();
        for relation in rel.iter() {
            result = match relation {
                ElementRelation::Parent => ElementRef::wrap(result.parent().unwrap()).unwrap(),
                ElementRelation::Child(i) => {
                    if *i < 0 {
                        ElementRef::wrap(result.last_child().unwrap()).unwrap()
                    } else if *i == 0 {
                        ElementRef::wrap(result.first_child().unwrap()).unwrap()
                    } else {
                        let mut sib = ElementRef::wrap(result.first_child().unwrap()).unwrap();
                        for _ in 0..*i {
                            sib = ElementRef::wrap(sib.next_sibling().unwrap()).unwrap();
                        }
                        sib //for j in
                    } //if then else i
                } //ElementRelation::Child
                ElementRelation::Sibling(i) => {
                    let mut sib = ElementRef::wrap(result.next_sibling().unwrap()).unwrap();
                    for _ in 0..*i {
                        sib = ElementRef::wrap(sib.next_sibling().unwrap()).unwrap();
                    }
                    sib
                } //ElementRelation::Sibling
            }; //match relation
               // println!("result: {}\n\n", result.inner_html());
        } //for relation
        result
    } //fn navigate_relation
}

#[cfg(test)]
mod tests {
    use super::CoinMarketCapScrapper;
    #[test]
    // fn test_get_inner_html_from_element() {
    //     let source = r#"<div class="sc-16r8icm-0 kXPxnI priceSection___3kA4m"><h1 class="priceHeading___2GB9O">BakeryToken Price<!-- --> <small>(<!-- -->BAKE<!-- -->)</small></h1><div class="sc-16r8icm-0 kXPxnI priceTitle___1cXUG"><div class="priceValue___11gHJ">$1.21</div><span style="background-color:var(--down-color);color:#fff;padding:5px 10px;border-radius:8px;font-size:14px;font-weight:600" class="sc-1v2ivon-0 gClTFY"><span class="icon-Caret-down"></span>18.45<!-- -->%</span></div><div class="sc-16r8icm-0 kXPxnI alternatePrices___1M7uY"><p class="sc-10nusm4-0 bspaAT">0.00002477 BTC<span style="color:var(--down-color);padding:0;border-radius:8px" class="sc-1v2ivon-0 gClTFY"><span class="icon-Caret-down"></span>15.53<!-- -->%</span></p><p class="sc-10nusm4-0 bspaAT">0.0007738 ETH<span style="color:var(--down-color);padding:0;border-radius:8px" class="sc-1v2ivon-0 gClTFY"><span class="icon-Caret-down"></span>15.52<!-- -->%</span></p></div><div class="sc-16r8icm-0 kXPxnI sliderSection___tjBoJ"><div class="sc-16r8icm-0 hfoyRV nowrap___2C79N"><span class="highLowLabel___2bI-G">Low<!-- -->:</span><span class="highLowValue___GfyK7">$1.21</span></div><div class="sc-16r8icm-0 kXPxnI slider___2_uly"><span style="width:100%" class="sc-1hm9f3g-0 dmzjSD"><span style="width: 0.414647%;"><svg xmlns="http://www.w3.org/2000/svg" fill="currentColor" height="24px" width="24px" viewBox="0 0 24 24" class="sc-16r8icm-0 eZMaTl sc-1hm9f3g-1 cbEuhW"><path d="M18.0566 16H5.94336C5.10459 16 4.68455 14.9782 5.27763 14.3806L11.3343 8.27783C11.7019 7.90739 12.2981 7.90739 12.6657 8.27783L18.7223 14.3806C19.3155 14.9782 18.8954 16 18.0566 16Z"></path></svg></span></span></div><div class="sc-16r8icm-0 ejXAFe nowrap___2C79N"><span class="highLowLabel___2bI-G">High<!-- -->:</span><span class="highLowValue___GfyK7">$1.54</span></div><div class="sc-16r8icm-0 ejphsb namePillBase___AZ1aa" display="inline-block">24h<svg xmlns="http://www.w3.org/2000/svg" fill="none" height="12" width="12" viewBox="0 0 24 24" style="height:10px" class="sc-16r8icm-0 cqmVDB"><path d="M6 9L12 15L18 9" stroke="currentColor" stroke-width="2" stroke-miterlimit="10" stroke-linecap="round" stroke-linejoin="round"></path></svg></div></div><div class="priceInfoPopup___gpebJ "><span><img src="https://s2.coinmarketcap.com/static/img/coins/64x64/7064.png" height="24" width="24" alt="BAKE">&nbsp;&nbsp;<b>BakeryToken</b>&nbsp;<!-- -->BAKE</span><span><span class="price"><span>Price: </span>$1.21<!-- -->&nbsp;<span style="background-color:var(--down-color);color:#fff;padding:3px 10px;border-radius:8px" class="qe1dn9-0 RYkpI"><span class="icon-Caret-down"></span>18.45<!-- -->%</span></span><span class="sc-7f3up6-1 dtMKRz is-starred"><button class="sc-1ejyco6-0 eBGPbT sc-7pvt85-0 ccOrkS" style="width: auto; padding: 0px 8px;">Remove from Main Watchlist &nbsp;<span class="icon-Star-Filled"></span></button></span></span></div></div>"#;
    //     let regex = r#"<(div) class="(priceTitle_.*?)">.*?</div>"#;
    //     let result =
    //         html_parser::get_inner_html_from_element(regex, source, ElementRelation::Child(0));
    //     match result {
    //         Ok(c) => {
    //             assert_eq!(c.inner_html(),String::from()
    //         }
    //         Err(e) => panic!("no result"),
    //     }
    // }
    #[test]
    fn test_get_price_existing_symbol() {
        let mut scrapper = CoinMarketCapScrapper::new(String::from("./config/config.toml"));
        let (price, percentage) = scrapper.get_price("multi-collateral-dai").unwrap();
        assert_eq!(price, 1.0); //testing with stable coin. expected value is 1.0
        assert!(percentage < 0.2 && percentage > -0.2); //testing with a stable coin. Difference should be less than 0.2%
    }
    #[test]
    fn test_get_price_non_existing_symbol() {
        let mut scrapper = CoinMarketCapScrapper::new(String::from("./config/config.toml"));
        let result = scrapper.get_price("aseff");
        match result {
            Ok(_) => assert!(false, "expected an error"),
            Err(e) => assert!(true, format!("Error is {}", e)),
        };
    }
}
