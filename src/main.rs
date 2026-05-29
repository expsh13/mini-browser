mod css;
mod error;
mod html;
mod http;
mod layout;
mod url;
mod window;

use crate::error::BrowserError;
use crate::html::html_to_styled_text;
use crate::http::{fetch, response_body};
use crate::layout::layout_styled_text;
use crate::url::parse_url;
use crate::window::show_window;
use std::env;

#[derive(Debug, PartialEq, Eq)]
struct Config {
    raw: bool,
    url: String,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), BrowserError> {
    let config = parse_args(env::args().skip(1))?;

    let url = parse_url(&config.url)?;
    let response = fetch(&url)?;

    if config.raw {
        print!("{response}");
    } else {
        let body = response_body(&response);
        let text = html_to_styled_text(body);
        let runs = layout_styled_text(&text);
        show_window(&runs)?;
    }

    Ok(())
}

fn parse_args(args: impl IntoIterator<Item = String>) -> Result<Config, BrowserError> {
    let mut raw = false;
    let mut url = None;

    for arg in args {
        if arg == "--raw" {
            raw = true;
        } else if url.is_none() {
            url = Some(arg);
        } else {
            return Err(BrowserError::InvalidUrl(usage()));
        }
    }

    let url = url.ok_or_else(|| BrowserError::InvalidUrl(usage()))?;

    Ok(Config { raw, url })
}

fn usage() -> String {
    "usage: mini-browser [--raw] http://example.org/".into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_url_argument() {
        let config = parse_args(["http://example.org/".to_string()]).unwrap();

        assert_eq!(
            config,
            Config {
                raw: false,
                url: "http://example.org/".into(),
            }
        );
    }

    #[test]
    fn parses_raw_option() {
        let config = parse_args(["--raw".to_string(), "http://example.org/".to_string()]).unwrap();

        assert_eq!(
            config,
            Config {
                raw: true,
                url: "http://example.org/".into(),
            }
        );
    }
}
