mod token;

pub use token::{Token, TokenType};

pub struct Lexer {
    input: String,
    idx: usize,
}

impl Lexer {
    pub fn new(input: String) -> Self {
        Self { input, idx: 0 }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_until(|c| c.is_whitespace() && c != '\0');
        let start = self.idx;

        match self.current_char() {
            ('a'..='z') | ('A'..='Z') | '_' => {
                self.skip_until(|c| !c.is_whitespace() && c != '\0' && c != ';');
                let end = self.idx;
                self.consume();
                let slice = &self.input[start..end];
                let token_type = TokenType::from(slice);
                Token::new(token_type, Some(slice.to_owned()))
            }
            '"' => {
                self.consume();
                let start = self.idx;
                self.skip_until(|c| c != '"');
                let end = self.idx;
                self.consume();
                Token::new(TokenType::Text, Some((&self.input[start..end]).to_owned()))
            }
            ';' => {
                self.consume();
                Token::new(TokenType::Semicolon, None)
            }
            '\0' => Token::new(TokenType::Eof, None),
            _ => {
                let start = self.idx;
                self.skip_until(|c| !c.is_whitespace() && c != '\0');
                let end = self.idx;
                Token::new(
                    TokenType::Illegal,
                    Some((&self.input[start..end]).to_owned()),
                )
            }
        }
    }

    fn skip_until<F>(&mut self, until: F)
    where
        F: Fn(char) -> bool,
    {
        while until(self.current_char()) {
            self.consume()
        }
    }

    fn consume(&mut self) {
        self.idx += 1;
    }

    fn current_char(&self) -> char {
        self.input.chars().nth(self.idx).unwrap_or('\0')
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn given_single_words_should_be_tokenize_correctly() {
        let cases = [
            ("with", TokenType::With),
            ("message", TokenType::Message),
            ("foo", TokenType::Name),
            ("_foo", TokenType::Name),
            ("\"bar baz\"", TokenType::Text),
            ("publisher", TokenType::Publisher),
            ("subscribe", TokenType::Subscribe),
            ("group", TokenType::Group),
            (";", TokenType::Semicolon),
            ("123", TokenType::Illegal),
            ("_123", TokenType::Name),
        ];
        for (input, expected) in cases {
            let mut lexer = Lexer::new(input.into());
            let tok = lexer.next_token();
            assert_eq!(
                tok.token_type(),
                expected,
                "with input {input} expected {expected:?} but got {:?}",
                tok.token_type()
            );
        }
    }

    #[test]
    fn given_sequence_should_be_tokenize_correctly() {
        let input = "with foo _foo 
        \"bar baz\" publisher 
        group ; 123 message _123"
            .to_string();
        let expecteds = [
            TokenType::With,
            TokenType::Name,
            TokenType::Name,
            TokenType::Text,
            TokenType::Publisher,
            TokenType::Group,
            TokenType::Semicolon,
            TokenType::Illegal,
            TokenType::Message,
            TokenType::Name,
        ];
        let mut lexer = Lexer::new(input);
        for expected in expecteds {
            let tok = lexer.next_token();
            assert_eq!(
                expected,
                tok.token_type(),
                "expected {expected:?} but got {:?}",
                tok.token_type()
            );
        }
    }

    #[test]
    fn given_sequence_value_should_be_ok() {
        let input = "subscribe foo with group bar".to_string();
        let expecteds = [
            Token::new(TokenType::Subscribe, Some("subscribe".to_owned())),
            Token::new(TokenType::Name, Some("foo".to_owned())),
            Token::new(TokenType::With, Some("with".to_owned())),
            Token::new(TokenType::Group, Some("group".to_owned())),
            Token::new(TokenType::Name, Some("bar".to_owned())),
        ];
        let mut lexer = Lexer::new(input);
        for expected in expecteds {
            let tok = lexer.next_token();
            assert_eq!(expected, tok, "expected {expected:?} but got {tok:?}");
        }
    }
}
