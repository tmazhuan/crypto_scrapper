pub mod config;
pub mod html;

use config::ConfigObject;
use html::ElementRelation::Child;
use html::{ElementRelation, HtmlParser, ParseError};
use regex::Regex;
use std::fmt;
use std::fmt::Display;
use std::sync::Arc;

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
        // self.runtime.block_on(self.html_parser.close_connection());
    }
}

impl CoinMarketCapScrapper {
    pub fn new(config_file_location: String) -> Result<CoinMarketCapScrapper, ParseError> {
        let rt = tokio::runtime::Runtime::new().unwrap();
        // let hparser = rt.block_on(async { HtmlParser::new(300).await });
        // let hparser = match rt.block_on(async { HtmlParser::new(300).await }) {
        //     Ok(p) => p,
        //     Err(e) => return Err(e),
        // };
        Ok(CoinMarketCapScrapper {
            cfg: ConfigObject::new(config_file_location),
            html_parser: match rt.block_on(async { HtmlParser::new(300).await }) {
                Ok(p) => p,
                Err(e) => return Err(e),
            },
            runtime: rt,
        })
    }

    pub fn get_details(&mut self, symbol: &str) -> Result<String, ParseError> {
        println!("in CoinMarketCapScrapper::new_get_details");

        let url = format!("https://coinmarketcap.com/currencies/{}/", symbol);
        let c = Arc::clone(&self.html_parser.cache);
        let html = match self.runtime.block_on(html::get_html(c, &url, false, true)) {
            Ok(html) => html,
            Err(err) => return Err(err),
        };
        let mut what_is_inner = match html::get_inner_html_from_element(
            &self.cfg.configuration.what_is_regex,
            &html,
            vec![vec![ElementRelation::Parent]],
        ) {
            Ok(s) => String::from(&s[0]),
            Err(_) => {
                match html::get_inner_html_from_element(
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
        let s = &self.cfg.configuration.symbols;
        self.get_prices(s)
    }
    pub fn get_prices(&self, symbols: &Vec<String>) -> Result<Vec<PriceResult>, ParseError> {
        let x = self.runtime.block_on(async {
            let mut handle_vector = Vec::new();
            let mut result = Vec::new();
            {
                for s in symbols {
                    let c = Arc::clone(&self.html_parser.cache);
                    let symbol = String::from(s);
                    let r = tokio::spawn(async move {
                        let url = format!("https://coinmarketcap.com/currencies/{}/", symbol);
                        let html = html::get_html(c, &url, true, false).await.unwrap();
                        CoinMarketCapScrapper::parse_price(html, url, symbol).unwrap()
                    });
                    handle_vector.push(r);
                }
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
        let price = match html::get_inner_html_from_element(
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
        let percentage = match html::get_inner_html_from_element(
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
    pub fn get_price(&mut self, symbol: &str) -> Result<PriceResult, ParseError> {
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
        let c = Arc::clone(&self.html_parser.cache);
        let html = match self.runtime.block_on(html::get_html(c, &url, false, true)) {
            Ok(html) => html,
            Err(err) => return Err(err),
        };
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
            let inner_source = html::get_inner_html_from_element(regex, &html, rel_source)?;
            let inner_pairs = html::get_inner_html_from_element(regex, &html, rel_pairs)?;
            let inner_price = html::get_inner_html_from_element(regex, &html, rel_price)?;
            let inner_vol = html::get_inner_html_from_element(regex, &html, rel_vol)?;
            let inner_vol_perc = html::get_inner_html_from_element(regex, &html, rel_vol_perc)?;
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
