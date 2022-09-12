use std::fmt::Display;

use bytes::Bytes;

#[derive(Eq, Clone, Copy, Debug)]
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
    Len(usize),
    Binary,
}

impl PartialEq for TokenType {
    fn eq(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }
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
            Self::With => "with".to_owned(),
            Self::Group => "group".to_owned(),
            Self::Publisher => "publisher".to_owned(),
            Self::Subscribe => "subscribe".to_owned(),
            Self::Message => "message".to_owned(),
            Self::Ok => "ok".to_owned(),
            Self::Name => "name".to_owned(),
            Self::Eof => "eof".to_owned(),
            Self::Semicolon => ";".to_owned(),
            Self::Illegal => "illegal".to_owned(),
            Self::Binary => "binary".to_owned(),
            Self::Len(len) => format!("len {}", len),
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
