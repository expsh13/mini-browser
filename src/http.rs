use crate::error::BrowserError;
use crate::url::Url;
use std::io::{Read, Write};
use std::net::TcpStream;

pub fn fetch(url: &Url) -> Result<String, BrowserError> {
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

pub fn response_body(response: &str) -> &str {
    response
        .split_once("\r\n\r\n")
        .or_else(|| response.split_once("\n\n"))
        .map(|(_, body)| body)
        .unwrap_or(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;
    use std::thread;

    #[test]
    fn extracts_response_body() {
        let response = "HTTP/1.0 200 OK\r\nContent-Type: text/html\r\n\r\n<html>Hello</html>";

        assert_eq!(response_body(response), "<html>Hello</html>");
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

        assert_eq!(crate::html::html_to_text(response_body(&response)), "Hello");
    }
}
