use std::fmt;

#[derive(Debug)]
pub enum BrowserError {
    InvalidUrl(String),
    Io(std::io::Error),
    Window(minifb::Error),
}

impl fmt::Display for BrowserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BrowserError::InvalidUrl(message) => write!(f, "{message}"),
            BrowserError::Io(error) => write!(f, "{error}"),
            BrowserError::Window(error) => write!(f, "{error}"),
        }
    }
}

impl From<std::io::Error> for BrowserError {
    fn from(error: std::io::Error) -> Self {
        BrowserError::Io(error)
    }
}

impl From<minifb::Error> for BrowserError {
    fn from(error: minifb::Error) -> Self {
        BrowserError::Window(error)
    }
}
