use crate::error::BrowserError;

#[derive(Debug, PartialEq, Eq)]
pub struct Url {
    pub host: String,
    pub port: u16,
    pub path: String,
}

pub fn parse_url(input: &str) -> Result<Url, BrowserError> {
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
