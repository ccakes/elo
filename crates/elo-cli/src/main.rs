use elo_core::{RateStore, Session};
use std::io::{self, BufRead};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let mut locale_id: Option<String> = None;
    let mut expr_args: Vec<String> = Vec::new();
    let mut skip_next = false;

    for (i, arg) in args.iter().skip(1).enumerate() {
        if skip_next {
            skip_next = false;
            continue;
        }
        if arg == "--locale" || arg == "-l" {
            if let Some(val) = args.get(i + 2) {
                locale_id = Some(val.clone());
                skip_next = true;
            }
        } else if arg.starts_with("--locale=") {
            locale_id = Some(arg.trim_start_matches("--locale=").to_string());
        } else {
            expr_args.push(arg.clone());
        }
    }

    let _locale = match locale_id {
        Some(id) => elo_core::Locale::from_identifier(&id),
        None => elo_core::Locale::from_system(),
    };

    let rates = RateStore::load();

    if !expr_args.is_empty() {
        // Single expression mode
        let expr = expr_args.join(" ");
        let mut session = Session::with_rates(rates);
        let result = session.eval_line(&expr);
        if !result.value.is_empty() {
            println!("{}", result.display);
        }
    } else {
        // Interactive / pipe mode
        let stdin = io::stdin();
        let mut session = Session::with_rates(rates);
        for line in stdin.lock().lines() {
            match line {
                Ok(input) => {
                    let result = session.eval_line(&input);
                    if !result.value.is_empty() {
                        println!("{}", result.display);
                    }
                }
                Err(_) => break,
            }
        }
    }
}
