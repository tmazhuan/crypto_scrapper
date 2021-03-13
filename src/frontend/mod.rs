use clipboard_win::{formats, Clipboard, Setter};
use crypto_scrapper::CoinMarketCapScrapper;

///Funtion to copy the String stored in `value`to the System Clipboard.
pub fn to_clip(value: String) {
    let _clip = Clipboard::new_attempts(10).unwrap();
    formats::Unicode.write_clipboard(&value).unwrap();
}
///Reads the input from the `stdin` and returns the trimmed version as a String
fn read_std_input() -> String {
    let mut line = String::new();
    std::io::stdin().read_line(&mut line).unwrap();
    String::from(line.trim())
}

///Enumarates and formats the content of the `symbols`vector and returns the resulting String
fn get_indexed_symbols(symbols: &Vec<String>) -> String {
    let mut i = 1;
    symbols.iter().fold(String::from(""), |mut s, x| {
        s.push_str(&format!("{}. {}\n", i, x));
        i += 1;
        s
    })
}

///The main menu loop of the Commandline Interface
pub fn cli_menu(mut scrapper: CoinMarketCapScrapper) {
    let mut level = 0;
    let mut exit = false;
    let mut input: String;
    while !exit {
        if level == 0 {
            let level_index = "1. Get all Prices for Excel Update\n2. What is ...\n3. Marekts\n4. Config\n5. Exit";
            let menu_level_index = format!("Level {}\n{}", level, level_index);
            println!("{}", menu_level_index);
            input = read_std_input();
            if input == "1" {
                let price_result = scrapper.get_all_prices();
                if let Err(e) = price_result {
                    println!("{}", e);
                    continue;
                };
                let prices = price_result.unwrap();
                let result = prices.iter().fold(String::new(), |clip, x| {
                    format!("{}{}\n", clip, x.to_string())
                });
                to_clip(result);
                println!(
                    "{}",
                    prices
                        .iter()
                        .fold(String::new(), |clip, x| format!("{}{}\n", clip, x))
                );
                println!("Prices also copied to Clipboard")
            } else if input == "2" {
                level = 2;
            //whatis
            } else if input == "3" {
                level = 3;
            } else if input == "4" {
                level = 4;
            } else if input == "5" {
                exit = true;
            } else {
                println!("Not a proper option. Please select again.");
            }
        } else if level == 2 {
            let level_index = "1. By symbol\n2. By index\n3. Back to Main Menu";
            let menu_level_index = format!("Level {}\n{}", level, level_index);
            println!("{}", menu_level_index);
            input = read_std_input();
            if input == "1" {
                println!("Enter symbol:");
                input = read_std_input();
                if let Ok(r) = scrapper.get_details(&input) {
                    println!("{}", r);
                } else {
                    println!("Symbol not recognized. Try again.");
                }
            } else if input == "2" {
                let symbols = scrapper.cfg.get_symbols();
                let i = symbols.len();
                println!("{}", get_indexed_symbols(&symbols));
                println!(
                    "Which item you want to delete? Enter \"{}\" to go back to the menu without getting details.",
                    i+1
                );
                let j = read_std_input().parse::<usize>().unwrap();
                if j <= i {
                    if let Ok(r) = scrapper.get_details(symbols.get(j - 1).unwrap()) {
                        println!("{}", r);
                    } else {
                        println!("Symbol not recognized. Try again.");
                    }
                }
            } else if input == "3" {
                level = 0;
            }
        } else if level == 3 {
            let level_index = "1. By symbol\n2. By index\n3. Back to Main Menu";
            let menu_level_index = format!("Level {}\n{}", level, level_index);
            println!("{}", menu_level_index);
            input = read_std_input();
            if input == "1" {
                println!("Enter symbol:");
                input = read_std_input();
                if let Ok(r) = scrapper.get_market_data(&input, 3) {
                    r.iter().for_each(|r| println!("{}", r));
                } else {
                    println!("Symbol not recognized. Try again.");
                }
            } else if input == "2" {
                let symbols = scrapper.cfg.get_symbols();
                let i = symbols.len();
                println!("{}", get_indexed_symbols(&symbols));
                println!(
                    "For which item you want the market data? Enter \"{}\" to go back to the menu without getting details.",
                    i+1
                );
                let j = read_std_input().parse::<usize>().unwrap();
                if j <= i {
                    if let Ok(r) = scrapper.get_market_data(symbols.get(j - 1).unwrap(), 3) {
                        r.iter().for_each(|r| println!("{}", r));
                    } else {
                        println!("Symbol not recognized. Try again.");
                    }
                }
            } else if input == "3" {
                level = 0;
            }
        } else if level == 4 {
            let level_index =
                "1. Show Symbols\n2. Add Symbol\n3. Delete Symbol\n4. Store config\n5. Back to Main Menu";
            let menu_level_index = format!("Level {}\n{}", level, level_index);
            println!("{}", menu_level_index);
            input = read_std_input();
            if input == "1" {
                let symbols = scrapper.cfg.get_symbols();
                println!("{}", get_indexed_symbols(&symbols));
            } else if input == "2" {
                println!("Enter symbol as it appears on CoinMarketCap-Addressbar. Enter \"back\" to return to menu: ");
                let s = read_std_input();
                if s.trim() != "back" {
                    scrapper.cfg.add_symbol(s);
                    println!("Added symbol to config file.");
                }
            } else if input == "3" {
                let symbols = scrapper.cfg.get_symbols();
                let i = symbols.len();
                println!("{}", get_indexed_symbols(&symbols));
                println!(
                    "Which item you want to delete? Enter \"{}\" to go back to the menu without getting details.",
                    i+1
                );
                let j = read_std_input().parse::<usize>().unwrap();
                if j <= i {
                    println!("{} removed.", scrapper.cfg.delete_symbol(j - 1));
                }
            } else if input == "4" {
                match scrapper.cfg.store() {
                    Ok(_) => println!("Stored Config-File sucessfully."),
                    Err(e) => println!("Error happend while storing config:\n{}", e),
                }
            } else if input == "5" {
                level = 0;
            }
        }
    }
}
