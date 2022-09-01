#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum TokenType {
    With,
    Name,
    Group,
    Illegal,
    Message,
    Text,
    Publisher,
    Subscribe,
    Semicolon,
    Eof,
}

impl From<&str> for TokenType {
    fn from(input: &str) -> Self {
        match &input.to_lowercase()[..] {
            "with" => Self::With,
            "group" => Self::Group,
            "publisher" => Self::Publisher,
            "subscribe" => Self::Subscribe,
            "message" => Self::Message,
            _ => Self::Name,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Token {
    token_type: TokenType,
    value: Option<String>,
}

impl Token {
    pub fn new(token_type: TokenType, value: Option<String>) -> Self {
        Self { token_type, value }
    }

    pub fn token_type(&self) -> TokenType {
        self.token_type
    }

    pub fn value(&self) -> Option<String> {
        self.value.clone()
    }
}
