pub mod config;

use config::ConfigObject;
use html_parser::ElementRelation::Child;
use html_parser::{ElementRelation, HtmlParser, ParseError};
use regex::Regex;
use std::fmt;
use std::fmt::Display;

pub struct PriceResult {
    symbol: String,
    price: f64,
    change: f64,
}

impl Display for PriceResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {} ({})", self.symbol, self.price, self.change)
    }
}
impl PriceResult {
    pub fn to_string(&self) -> String {
        String::from(format!("{}\t{}", self.symbol, self.price))
    }
}
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
        result.push_str(&self.get_spaces(column_widht - self.source.len()));
        result.push_str(&self.pair);
        result.push_str(&self.get_spaces(column_widht - self.pair.len()));
        let price = format!("{}", (self.price));
        result.push_str(&price);
        result.push_str(&self.get_spaces(column_widht - price.len()));
        let vol = format!("{}", (self.volume));
        result.push_str(&vol);
        result.push_str(&self.get_spaces(column_widht - vol.len()));
        let vol_per = format!("{}", (self.volume_percent));
        result.push_str(&vol_per);
        write!(f, "{}", result)
    }
}

pub struct CoinMarketCapScrapper {
    pub cfg: ConfigObject,
    html_parser: HtmlParser,
    runtime: tokio::runtime::Runtime,
}

impl Drop for CoinMarketCapScrapper {
    fn drop(&mut self) {
        self.runtime.block_on(self.html_parser.close_connection());
    }
}

impl CoinMarketCapScrapper {
    pub fn new(config_file_location: String) -> Result<CoinMarketCapScrapper, ParseError> {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let hparser = rt.block_on(async { HtmlParser::new(300).await });
        let hparser = match hparser {
            Ok(p) => p,
            Err(e) => return Err(e),
        };
        Ok(CoinMarketCapScrapper {
            cfg: ConfigObject::new(config_file_location),
            html_parser: hparser,
            runtime: rt,
        })
    }
    pub fn get_details(&mut self, symbol: &str) -> Result<String, ParseError> {
        println!("in CoinMarketCapScrapper::new_get_details");

        let url = format!("https://coinmarketcap.com/currencies/{}/", symbol);
        let html = self
            .runtime
            .block_on(self.html_parser.get_html_source_with_script(&url, false))
            .unwrap();
        let mut what_is_inner = match HtmlParser::get_inner_html_from_element(
            &self.cfg.configuration.what_is_regex,
            &html,
            vec![vec![ElementRelation::Parent]],
        ) {
            Ok(s) => String::from(&s[0]),
            Err(_) => {
                match HtmlParser::get_inner_html_from_element(
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

    pub fn get_all_prices(&self) -> Result<Vec<PriceResult>, ParseError> {
        self.get_prices(&self.cfg.configuration.symbols)
    }
    pub fn get_prices(&self, symbols: &Vec<String>) -> Result<Vec<PriceResult>, ParseError> {
        let x = self.runtime.block_on(async {
            let mut handle_vector = Vec::new();
            let mut result = Vec::new();
            for s in symbols {
                let symbol = String::from(s);
                let r = tokio::spawn(async move {
                    let url = format!("https://coinmarketcap.com/currencies/{}/", symbol);
                    let html = HtmlParser::get_html_source_no_script(&url).await.unwrap();
                    CoinMarketCapScrapper::parse_price(html, url, symbol).unwrap()
                });
                handle_vector.push(r);
            }
            for h in handle_vector {
                result.push(h.await.unwrap());
            }
            result
        });
        Ok(x)
    }
    fn parse_price(html: String, url: String, symbol: String) -> Result<PriceResult, ParseError> {
        let reg_price_section = r#"(div) (class=".{1,20}priceTitle__.{1,20}")>"#;
        //price
        let price = match HtmlParser::get_inner_html_from_element(
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
        let percentage = match HtmlParser::get_inner_html_from_element(
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
        let change = match cap {
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
        Ok(PriceResult {
            symbol,
            price,
            change,
        })
    }
    pub fn get_price(&self, symbol: &str) -> Result<PriceResult, ParseError> {
        let mut result = self.get_prices(&vec![String::from(symbol)]).unwrap();
        Ok(result.pop().unwrap())
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
        let url = format!("https://coinmarketcap.com/currencies/{}/markets", symbol);
        let html = self
            .runtime
            .block_on(self.html_parser.get_html_source_with_script(&url, false))
            .unwrap();
        for i in 0..number_of_results {
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
            let inner_source = HtmlParser::get_inner_html_from_element(regex, &html, rel_source)?;
            let inner_pairs = HtmlParser::get_inner_html_from_element(regex, &html, rel_pairs)?;
            let inner_price = HtmlParser::get_inner_html_from_element(regex, &html, rel_price)?;
            let inner_vol = HtmlParser::get_inner_html_from_element(regex, &html, rel_vol)?;
            let inner_vol_perc =
                HtmlParser::get_inner_html_from_element(regex, &html, rel_vol_perc)?;
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
    use std::time::{Duration, Instant};

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
        cache: HashMap<String, CacheEntry>,
        refresh_after: Duration,
    }

    pub struct CacheEntry {
        html: String,
        timestamp: Instant,
    }

    impl HtmlParser {
        pub async fn new(refresh_after: u64) -> Result<HtmlParser, ParseError> {
            let c = match ClientBuilder::native()
                .connect("http://localhost:9515")
                .await
            {
                Ok(c) => c,
                Err(e) => {
                    println!("{}", e);
                    return Err(ParseError::new(String::from(
                        "Make sure Chromedriver is started.",
                    )));
                }
            };
            return Ok(HtmlParser {
                client: c,
                cache: HashMap::new(),
                refresh_after: Duration::from_secs(refresh_after),
            });
        }

        pub async fn close_connection(&mut self) {
            self.client.close().await.unwrap();
        }

        pub async fn get_html_source_no_script(url: &str) -> Result<String, ParseError> {
            Ok(reqwest::get(url).await.unwrap().text().await.unwrap())
        }

        ///Loads the html source of the given url after having executed javascript functionality on the page. If the corresponding page is
        /// is alsredy stored in the cache and the `reload`parameter is set to false and the cache is not outdate, the result is loaded from
        /// the cache, otherwise it is loaded online.
        pub async fn get_html_source_with_script(
            &mut self,
            url: &str,
            reload: bool,
        ) -> Result<String, ParseError> {
            //first lets check if we have the source already in the cache if we don't need to reload
            //check if data is alread in the cache
            let in_cache = self.cache.contains_key(url);
            if !reload && in_cache || (reload && in_cache) {
                let entry = self.cache.get(url).unwrap();
                if !reload || entry.timestamp.elapsed().as_secs() < self.refresh_after.as_secs() {
                    println!("Getting value from cache");
                    return Ok(String::from(&entry.html));
                } else {
                    println!("Outdated value in cache. Reloading value");
                }
            };
            println!("loading URL and store it in cache");
            self.client.goto(url).await.unwrap();
            let source = self.client.source().await.unwrap();
            self.cache.insert(
                String::from(url),
                CacheEntry {
                    html: String::from(&source),
                    timestamp: Instant::now(),
                },
            );
            Ok(source)
        }
        pub fn get_inner_html_from_element(
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
            let selector = Selector::parse(&format!("{}[{}]", tag, attribute)).unwrap();
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

    ///Navigate
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
        } //for relation
        result
    } //fn navigate_relation
}

#[cfg(test)]
mod tests {
    use super::CoinMarketCapScrapper;
    #[test]
    fn test_get_price_existing_symbol() {
        let scrapper = CoinMarketCapScrapper::new(String::from("./config/config.toml")).unwrap();
        let result = scrapper.get_price("multi-collateral-dai").unwrap();
        assert_eq!(result.price, 1.0); //testing with stable coin. expected value is 1.0
        assert!(result.change < 0.2 && result.change > -0.2); //testing with a stable coin. Difference should be less than 0.2%
    }
    #[test]
    fn test_get_price_non_existing_symbol() {
        let scrapper = CoinMarketCapScrapper::new(String::from("./config/config.toml")).unwrap();
        let result = scrapper.get_price("aseff");
        match result {
            Ok(_) => assert!(false, "expected an error"),
            Err(e) => assert!(true, format!("Error is {}", e)),
        };
    }
}
