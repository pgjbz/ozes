use std::fmt::Display;

use bytes::Bytes;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum TokenType {
    Ok,
    Eof,
    With,
    Name,
    Group,
    Illegal,
    Message,
    Publisher,
    Subscribe,
    Semicolon,
    Binary,
}

impl From<&[u8]> for TokenType {
    fn from(input: &[u8]) -> Self {
        match &input.to_ascii_lowercase()[..] {
            b"with" => Self::With,
            b"group" => Self::Group,
            b"publisher" => Self::Publisher,
            b"subscribe" => Self::Subscribe,
            b"message" => Self::Message,
            b"ok" => Self::Ok,
            _ => Self::Name,
        }
    }
}

impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::With => "with",
            Self::Group => "group",
            Self::Publisher => "publisher",
            Self::Subscribe => "subscribe",
            Self::Message => "message",
            Self::Ok => "ok",
            Self::Name => "name",
            Self::Eof => "eof",
            Self::Semicolon => ";",
            Self::Illegal => "illegal",
            Self::Binary => "binary",
        };
        write!(f, "{}", name)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Token {
    token_type: TokenType,
    value: Option<Bytes>,
}

impl Token {
    pub fn new(token_type: TokenType, value: Option<Bytes>) -> Self {
        Self { token_type, value }
    }

    pub fn token_type(&self) -> TokenType {
        self.token_type
    }

    pub fn value(&self) -> Option<Bytes> {
        self.value.clone()
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}: '{}']",
            self.token_type,
            if let Some(ref result) = self.value {
                String::from_utf8_lossy(result)
            } else {
                String::from_utf8_lossy(b"empty value")
            }
        )
    }
}
