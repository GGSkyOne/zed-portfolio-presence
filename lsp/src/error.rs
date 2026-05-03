use std::{fmt, string::FromUtf8Error};

#[derive(Debug)]
pub enum PresenceError {
    Http(String),
    Config(String),
    Io(std::io::Error),
    JsonParse(serde_json::Error),
    UrlDecode(FromUtf8Error),
}

impl fmt::Display for PresenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PresenceError::Http(msg) => write!(f, "HTTP error: {msg}"),
            PresenceError::Config(msg) => write!(f, "Config error: {msg}"),
            PresenceError::Io(err) => write!(f, "IO error: {err}"),
            PresenceError::JsonParse(err) => write!(f, "JSON parse error: {err}"),
            PresenceError::UrlDecode(err) => write!(f, "URL decode error: {err}"),
        }
    }
}

impl std::error::Error for PresenceError {}

impl From<std::io::Error> for PresenceError {
    fn from(err: std::io::Error) -> Self {
        PresenceError::Io(err)
    }
}

impl From<serde_json::Error> for PresenceError {
    fn from(err: serde_json::Error) -> Self {
        PresenceError::JsonParse(err)
    }
}

impl From<FromUtf8Error> for PresenceError {
    fn from(err: FromUtf8Error) -> Self {
        PresenceError::UrlDecode(err)
    }
}

impl From<reqwest::Error> for PresenceError {
    fn from(err: reqwest::Error) -> Self {
        PresenceError::Http(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, PresenceError>;
