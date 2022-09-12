mod token;

use bytes::Bytes;
pub use token::{Token, TokenType};

//TODO: use bytes to improve input data and support binary
pub struct Lexer {
    input: Bytes,
    idx: usize,
}

impl Lexer {
    pub fn new(input: Bytes) -> Self {
        Self { input, idx: 0 }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_until(|c| c.is_ascii_whitespace() && c != &0u8);
        let start = self.idx;
        //TODO: lex len token
        match self.current_char() {
            b'+' if self.next_char() == &b'l' => {
                self.consume();
                self.consume();
                let start = self.idx;
                self.skip_until(|c| c.is_ascii_digit() && c != &0u8);
                let end = self.idx;
                let number_slice = Bytes::copy_from_slice(&self.input[start..end]);

                let number_string = String::from_utf8_lossy(&number_slice);
                let number: usize = match number_string.parse() {
                    Ok(number) => number,
                    Err(_) => return Token::new(TokenType::Illegal, Some(number_slice)),
                };
                Token::new(TokenType::Len(number), None)
            }
            (b'a'..=b'z') | (b'A'..=b'Z') | b'_' => {
                self.skip_until(|c| {
                    (c.is_ascii_alphanumeric() || c == &b'_' || c == &b'.') && c != &0u8
                });
                let end = self.idx;
                self.consume();
                let slice = &self.input[start..end];
                let token_type = TokenType::from(slice);

                Token::new(token_type, Some(Bytes::copy_from_slice(slice)))
            }
            b';' => {
                self.consume();
                Token::new(TokenType::Semicolon, None)
            }
            b'#' => {
                self.consume();
                let token = Token::new(
                    TokenType::Binary,
                    Some(Bytes::copy_from_slice(&self.input[self.idx..])),
                );
                self.idx = self.input.len();
                token
            }
            0 => Token::new(TokenType::Eof, None),
            _ => {
                let start = self.idx;
                self.skip_until(|c| !c.is_ascii_whitespace() && c != &0u8);
                let end = self.idx;
                Token::new(
                    TokenType::Illegal,
                    Some(Bytes::copy_from_slice(&self.input[start..end])),
                )
            }
        }
    }

    fn skip_until<F>(&mut self, until: F)
    where
        F: Fn(&u8) -> bool,
    {
        while until(self.current_char()) {
            self.consume()
        }
    }

    fn consume(&mut self) {
        self.idx += 1;
    }

    fn next_char(&self) -> &u8 {
        self.input.get(self.idx + 1).unwrap_or(&0u8)
    }
    fn current_char(&self) -> &u8 {
        self.input.get(self.idx).unwrap_or(&0u8)
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
            ("publisher", TokenType::Publisher),
            ("subscribe", TokenType::Subscribe),
            ("group", TokenType::Group),
            (";", TokenType::Semicolon),
            ("123", TokenType::Illegal),
            ("_123", TokenType::Name),
            ("_ha_do_ken", TokenType::Name),
            ("_ha.do_ken", TokenType::Name),
            ("4+8", TokenType::Illegal),
            ("#4+8", TokenType::Binary),
            ("#4+8", TokenType::Binary),
            ("+l250", TokenType::Len(250)),
            ("la", TokenType::Name),
            ("+la", TokenType::Illegal),
            ("", TokenType::Eof),
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
        let input = Bytes::from_static(
            b"with foo _foo 
        publisher 
        group ; 123 message _123 4+8 pgjbz.dev +l250 love",
        );
        let expecteds = [
            TokenType::With,
            TokenType::Name,
            TokenType::Name,
            TokenType::Publisher,
            TokenType::Group,
            TokenType::Semicolon,
            TokenType::Illegal,
            TokenType::Message,
            TokenType::Name,
            TokenType::Illegal,
            TokenType::Name,
            TokenType::Len(250),
            TokenType::Name,
            TokenType::Eof,
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
        let input = Bytes::from_static(b"subscribe foo with group bar");
        let expecteds = [
            Token::new(TokenType::Subscribe, Some(Bytes::from_static(b"subscribe"))),
            Token::new(TokenType::Name, Some(Bytes::from_static(b"foo"))),
            Token::new(TokenType::With, Some(Bytes::from_static(b"with"))),
            Token::new(TokenType::Group, Some(Bytes::from_static(b"group"))),
            Token::new(TokenType::Name, Some(Bytes::from_static(b"bar"))),
        ];
        let mut lexer = Lexer::new(input);
        for expected in expecteds {
            let tok = lexer.next_token();
            assert_eq!(expected, tok, "expected {expected:?} but got {tok:?}");
        }
    }
}
