pub mod config;

use config::ConfigObject;
use html_parser::*;
use regex::Regex;
use scraper::{ElementRef, Html, Selector};

pub struct MarketResult {
    source: String,
    pair: String,
    price: f64,
    volume: u64,
    volume_dollar: u64,
    volume_percent: u32,
}

pub struct CoinMarketCapScrapper {
    cfg: ConfigObject,
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

    pub fn get_price(&self, symbol: &str) -> Result<(f64, f64), ParseError> {
        let url = format!("https://coinmarketcap.com/currencies/{}/", symbol);
        let html = reqwest::blocking::get(&url).unwrap().text().unwrap();
        let document = Html::parse_document(&html);
        let reg_price_section = r#"(div) (class=".{1,20}priceTitle__.{1,20}")>"#;
        // let r = r#"<(div) (class="priceValue_.*?")>.*?</div>"#;
        let el = html_parser::get_element_and_attribute(reg_price_section, &html);
        let (tag, attribute) = match el {
            Some(r) => (r.tag, r.attribute),
            None => {
                return Err(ParseError::new(format!(
                    "No Price found. Please check manually on {}",
                    url
                )))
            }
        };
        // println!("tag: {}, attribute: {}", tag, attribute);
        let selector = Selector::parse(&format!("{}[{}]", tag, attribute)).unwrap();
        let element = document.select(&selector).next().unwrap();
        // println!("InnerHTML: {}", element.inner_html());
        // element = element.next_sibling();
        let price_child = ElementRef::wrap(element.first_child().unwrap()).unwrap();
        // println!("priceChild: {:?}", price_child.value());
        let percentage_child = ElementRef::wrap(element.last_child().unwrap()).unwrap();
        // println!("InnerHTML: {}", percentage_child.inner_html());
        let selector = Selector::parse(r#"span[class="icon-Caret-up"]"#).unwrap();
        let fragment = Html::parse_fragment(&percentage_child.inner_html());
        let direction = fragment.select(&selector).next();
        let sign = match direction {
            Some(_) => 1.0,
            None => -1.0,
        };
        let per_regex = r#"</span>([0-9]+[.]?[0-9]*)<!-- -->%"#;
        let re = Regex::new(per_regex).unwrap();
        let source = percentage_child.inner_html();
        let cap = re.captures(&source);
        let percentage = (match cap {
            Some(c) => String::from(&c[1]).parse::<f64>().unwrap(),
            None => 0.0,
        }) * sign;

        // println!("sign is {}, amount is {}", sign, percentage);
        // let percentage_class = ElementRef::wrap(percentage_child.first_child().unwrap()).unwrap();
        Ok((
            price_child
                .inner_html()
                .replace("$", "")
                .parse::<f64>()
                .unwrap(),
            percentage,
        ))
    }

    #[allow(unused)]
    pub fn get_24h_performance(&self, symbol: &str) -> f64 {
        //
        0.0
    }

    // pub fn get_markets(&self, symbol: &str) -> Result<MarketResult, Error> {}

    #[allow(unused)]
    pub fn get_7d_performance(&self, symbol: &str) -> f64 {
        0.0
    }

    #[allow(unused)]
    fn get_performance(&self, symbol: &str, regex: String) -> f64 {
        0.0
    }
}

mod html_parser {
    use regex::Regex;
    // use scraper::{ElementRef, Html, Selector};
    use std::error::Error;
    use std::fmt;

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

    enum ElementRelation {
        Parent,
        Child(i32),
        Sibling(i32),
    }

    pub struct HtmlParseResult {
        pub tag: String,
        pub attribute: String,
    }

    pub fn get_element_and_attribute(regex: &str, source: &str) -> Option<HtmlParseResult> {
        let re = Regex::new(regex).unwrap();
        let cap = re.captures(source);
        match cap {
            Some(c) => Some(HtmlParseResult {
                tag: String::from(&c[1]),
                attribute: String::from(&c[2]),
            }),
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::html_parser;
    use super::CoinMarketCapScrapper;
    #[test]
    fn test_get_element_and_attribute() {
        let source = r#"<div class="sc-16r8icm-0 kXPxnI priceSection___3kA4m"><h1 class="priceHeading___2GB9O">BakeryToken Price<!-- --> <small>(<!-- -->BAKE<!-- -->)</small></h1><div class="sc-16r8icm-0 kXPxnI priceTitle___1cXUG"><div class="priceValue___11gHJ">$1.21</div><span style="background-color:var(--down-color);color:#fff;padding:5px 10px;border-radius:8px;font-size:14px;font-weight:600" class="sc-1v2ivon-0 gClTFY"><span class="icon-Caret-down"></span>18.45<!-- -->%</span></div><div class="sc-16r8icm-0 kXPxnI alternatePrices___1M7uY"><p class="sc-10nusm4-0 bspaAT">0.00002477 BTC<span style="color:var(--down-color);padding:0;border-radius:8px" class="sc-1v2ivon-0 gClTFY"><span class="icon-Caret-down"></span>15.53<!-- -->%</span></p><p class="sc-10nusm4-0 bspaAT">0.0007738 ETH<span style="color:var(--down-color);padding:0;border-radius:8px" class="sc-1v2ivon-0 gClTFY"><span class="icon-Caret-down"></span>15.52<!-- -->%</span></p></div><div class="sc-16r8icm-0 kXPxnI sliderSection___tjBoJ"><div class="sc-16r8icm-0 hfoyRV nowrap___2C79N"><span class="highLowLabel___2bI-G">Low<!-- -->:</span><span class="highLowValue___GfyK7">$1.21</span></div><div class="sc-16r8icm-0 kXPxnI slider___2_uly"><span style="width:100%" class="sc-1hm9f3g-0 dmzjSD"><span style="width: 0.414647%;"><svg xmlns="http://www.w3.org/2000/svg" fill="currentColor" height="24px" width="24px" viewBox="0 0 24 24" class="sc-16r8icm-0 eZMaTl sc-1hm9f3g-1 cbEuhW"><path d="M18.0566 16H5.94336C5.10459 16 4.68455 14.9782 5.27763 14.3806L11.3343 8.27783C11.7019 7.90739 12.2981 7.90739 12.6657 8.27783L18.7223 14.3806C19.3155 14.9782 18.8954 16 18.0566 16Z"></path></svg></span></span></div><div class="sc-16r8icm-0 ejXAFe nowrap___2C79N"><span class="highLowLabel___2bI-G">High<!-- -->:</span><span class="highLowValue___GfyK7">$1.54</span></div><div class="sc-16r8icm-0 ejphsb namePillBase___AZ1aa" display="inline-block">24h<svg xmlns="http://www.w3.org/2000/svg" fill="none" height="12" width="12" viewBox="0 0 24 24" style="height:10px" class="sc-16r8icm-0 cqmVDB"><path d="M6 9L12 15L18 9" stroke="currentColor" stroke-width="2" stroke-miterlimit="10" stroke-linecap="round" stroke-linejoin="round"></path></svg></div></div><div class="priceInfoPopup___gpebJ "><span><img src="https://s2.coinmarketcap.com/static/img/coins/64x64/7064.png" height="24" width="24" alt="BAKE">&nbsp;&nbsp;<b>BakeryToken</b>&nbsp;<!-- -->BAKE</span><span><span class="price"><span>Price: </span>$1.21<!-- -->&nbsp;<span style="background-color:var(--down-color);color:#fff;padding:3px 10px;border-radius:8px" class="qe1dn9-0 RYkpI"><span class="icon-Caret-down"></span>18.45<!-- -->%</span></span><span class="sc-7f3up6-1 dtMKRz is-starred"><button class="sc-1ejyco6-0 eBGPbT sc-7pvt85-0 ccOrkS" style="width: auto; padding: 0px 8px;">Remove from Main Watchlist &nbsp;<span class="icon-Star-Filled"></span></button></span></span></div></div>"#;
        let regex = r#"<(div) class="(priceValue_.*?)">.*?</div>"#;
        let result = html_parser::get_element_and_attribute(regex, source);
        match result {
            Some(c) => {
                assert_eq!(c.tag, "div");
                assert_eq!(c.attribute, "priceValue___11gHJ")
            }
            None => panic!("no result"),
        }
    }

    #[test]
    fn test_get_price_existing_symbol() {
        let scrapper = CoinMarketCapScrapper::new(String::from("./config/config.toml"));
        let (price, percentage) = scrapper.get_price("multi-collateral-dai").unwrap();
        assert_eq!(price, 1.0); //testing with stable coin. expected value is 1.0
        assert!(percentage < 0.2 && percentage > -0.2); //testing with a stable coin. Difference should be less than 0.2%
    }
    #[test]
    fn test_get_price_non_existing_symbol() {
        let scrapper = CoinMarketCapScrapper::new(String::from("./config/config.toml"));
        let result = scrapper.get_price("aseff");
        match result {
            Ok(_) => assert!(false, "expected an error"),
            Err(e) => assert!(true, format!("Error is {}", e)),
        };
    }
}
