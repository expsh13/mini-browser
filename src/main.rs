use std::env;
use std::fmt;
use std::io::{Read, Write};
use std::net::TcpStream;

#[derive(Debug, PartialEq, Eq)]
struct Url {
    host: String,
    port: u16,
    path: String,
}

#[derive(Debug, PartialEq, Eq)]
struct Config {
    raw: bool,
    url: String,
}

#[derive(Debug)]
enum BrowserError {
    InvalidUrl(String),
    Io(std::io::Error),
}

impl fmt::Display for BrowserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BrowserError::InvalidUrl(message) => write!(f, "{message}"),
            BrowserError::Io(error) => write!(f, "{error}"),
        }
    }
}

impl From<std::io::Error> for BrowserError {
    fn from(error: std::io::Error) -> Self {
        BrowserError::Io(error)
    }
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
        let text = html_to_text(body);
        println!("{text}");
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

fn parse_url(input: &str) -> Result<Url, BrowserError> {
    let rest = input
        .strip_prefix("http://")
        .ok_or_else(|| BrowserError::InvalidUrl("only http:// URLs are supported".into()))?;

    let (host_port, path) = match rest.split_once('/') {
        Some((host_port, path)) => (host_port, format!("/{path}")),
        None => (rest, "/".to_string()),
    };

    if host_port.is_empty() {
        return Err(BrowserError::InvalidUrl("URL host is empty".into()));
    }

    let (host, port) = match host_port.split_once(':') {
        Some((host, port)) => {
            let port = port
                .parse::<u16>()
                .map_err(|_| BrowserError::InvalidUrl("URL port is invalid".into()))?;
            (host.to_string(), port)
        }
        None => (host_port.to_string(), 80),
    };

    Ok(Url { host, port, path })
}

fn fetch(url: &Url) -> Result<String, BrowserError> {
    let mut stream = TcpStream::connect((url.host.as_str(), url.port))?;
    let request = format!(
        "GET {} HTTP/1.0\r\nHost: {}\r\nConnection: close\r\nUser-Agent: mini-browser\r\n\r\n",
        url.path, url.host
    );

    stream.write_all(request.as_bytes())?;

    let mut response = String::new();
    stream.read_to_string(&mut response)?;

    Ok(response)
}

fn response_body(response: &str) -> &str {
    response
        .split_once("\r\n\r\n")
        .or_else(|| response.split_once("\n\n"))
        .map(|(_, body)| body)
        .unwrap_or(response)
}

fn html_to_text(html: &str) -> String {
    let mut text = String::new();
    let mut in_tag = false;

    for char in html.chars() {
        match char {
            '<' => {
                in_tag = true;
                text.push(' ');
            }
            '>' => in_tag = false,
            _ if !in_tag => text.push(char),
            _ => {}
        }
    }

    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;
    use std::thread;

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

    #[test]
    fn parses_http_url() {
        let url = parse_url("http://example.org/articles/1").unwrap();

        assert_eq!(
            url,
            Url {
                host: "example.org".into(),
                port: 80,
                path: "/articles/1".into(),
            }
        );
    }

    #[test]
    fn parses_port() {
        let url = parse_url("http://localhost:3000/").unwrap();

        assert_eq!(url.host, "localhost");
        assert_eq!(url.port, 3000);
        assert_eq!(url.path, "/");
    }

    #[test]
    fn extracts_response_body() {
        let response = "HTTP/1.0 200 OK\r\nContent-Type: text/html\r\n\r\n<html>Hello</html>";

        assert_eq!(response_body(response), "<html>Hello</html>");
    }

    #[test]
    fn extracts_text_from_html() {
        let html = "<html><body><h1>Hello</h1><p>mini browser</p></body></html>";

        assert_eq!(html_to_text(html), "Hello mini browser");
    }

    #[test]
    fn fetches_html_over_tcp() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();

        thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0; 512];
            let _ = stream.read(&mut request).unwrap();
            stream
                .write_all(b"HTTP/1.0 200 OK\r\n\r\n<html><body>Hello</body></html>")
                .unwrap();
        });

        let url = Url {
            host: "127.0.0.1".into(),
            port,
            path: "/".into(),
        };

        let response = fetch(&url).unwrap();

        assert_eq!(html_to_text(response_body(&response)), "Hello");
    }
}
