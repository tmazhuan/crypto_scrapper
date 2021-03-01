pub mod config;

use config::ConfigObject;
use regex::Regex;
use scraper::{ElementRef, Html, Selector};
use std::error::Error;
use std::fmt;

pub struct CoinMarketCapScrapper {
    cfg: ConfigObject,
}

pub struct ParseError {
    details: String,
}

impl ParseError {
    fn new(details: String) -> ParseError {
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

impl CoinMarketCapScrapper {
    pub fn new(config_file_location: String) -> CoinMarketCapScrapper {
        CoinMarketCapScrapper {
            cfg: ConfigObject::new(config_file_location),
        }
    }
    pub fn get_details(&self, symbol: &str) -> Result<String, ParseError> {
        let url = format!("https://coinmarketcap.com/currencies/{}/", symbol);
        let html = reqwest::blocking::get(&url).unwrap().text().unwrap();
        let document = Html::parse_document(&html);
        let result = self.get_what_is_html_element(&html);
        let (t, v) = match result {
            Ok((t, v)) => (t, v),
            Err(e) => {
                return Err(ParseError::new(format!(
                    "Error ({}) when reading page. Check manually on {}",
                    e, url
                )));
            }
        };
        let selector = Selector::parse(&format!("{}[{}]", t, v)).unwrap();
        let mut element = document.select(&selector).next().unwrap();
        let mut iter = element.next_siblings();
        let mut next_element = false;
        let mut result = String::new();
        while !next_element {
            next_element = match iter.next() {
                Some(e) => {
                    element = ElementRef::wrap(e).unwrap();
                    let l = element.value().name();
                    if l == "h2" || l == "h3" {
                        true
                    } else {
                        // println!("{}", element.inner_html());
                        // clean_text_from_links(element.inner_html());
                        result.push_str(&format!("{}\n", element.inner_html()));
                        // element = ElementRef::wrap(iter.next().unwrap()).unwrap();
                        false
                    }
                }
                None => true,
            };
        }
        // println!("{}", self.cleanup_result_string(result));
        Ok(self.cleanup_result_string(result))
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
        String::from(text)
    }
    fn get_what_is_html_element(&self, html: &String) -> Result<(String, String), &'static str> {
        //<h2 id="what-is-cryptocom-coin-cro">What Is Crypto.com Coin [CRO]?</h2>
        // let re = Regex::new(r#"<(h2)\s{1}(.{1,10}="what-is-.*?")>.*?</h2>"#).unwrap();
        let re = Regex::new(&self.cfg.configuration.what_is_regex).unwrap();
        // let mut result = String::new();
        let capture = re.captures(&html);
        match capture {
            Some(cap) => {
                let t = &cap[1];
                let v = &cap[2];
                return Ok((String::from(t), String::from(v)));
            }
            None => {
                let re = Regex::new(&self.cfg.configuration.about_regex).unwrap();
                let capture = re.captures(&html);
                match capture {
                    Some(cap) => {
                        let t = &cap[1];
                        let v = &cap[2];
                        return Ok((String::from(t), String::from(v)));
                    }
                    None => return Err("Element not Found"),
                };
            }
        };
    }

    pub fn get_price(&self, symbol: &str) -> f64 {
        // let mut result = String::from("");
        let url = format!("https://coinmarketcap.com/currencies/{}/", symbol);

        let html = reqwest::blocking::get(&url).unwrap().text().unwrap();
        let re = Regex::new(r#"<div class="priceValue_.*?">(.*?)</div>"#).unwrap();
        // let mut price: String;
        for cap in re.captures_iter(&html) {
            let p = &cap[1];
            // let title = &cap[2];
            let price = String::from(p).replace(",", "").split_off(1);
            return price.parse::<f64>().unwrap();
        }
        return 0.0;
    }

    #[allow(unused)]
    pub fn get_24h_performance(&self, symbol: &str) -> f64 {
        0.0
    }

    #[allow(unused)]
    pub fn get_7d_performance(&self, symbol: &str) -> f64 {
        0.0
    }

    #[allow(unused)]
    fn get_performance(&self, symbol: &str, regex: String) -> f64 {
        0.0
    }
}
