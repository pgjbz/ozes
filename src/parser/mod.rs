use crate::lexer::{Lexer, Token, TokenType};

mod parse_error;
pub use parse_error::ParseError;

use self::parse_error::ParseResult;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Message(String),
    Publisher(String),
    Subscriber(String, String),
}

pub struct Parser {
    lexer: Lexer,
    current_tok: Token,
    next_tok: Token,
}

impl Parser {
    pub fn new(mut lexer: Lexer) -> Self {
        let (current_tok, next_tok) = (lexer.next_token(), lexer.next_token());
        Self {
            lexer,
            current_tok,
            next_tok,
        }
    }

    pub fn parse_commands(&mut self) -> ParseResult {
        let mut commands = vec![];
        while !self.current_token_is(TokenType::Eof) {
            if self.current_token_is(TokenType::Semicolon) {
                self.consume();
                if self.current_token_is(TokenType::Eof) {
                    break;
                }
            }
            let current_token = self.current_tok.token_type();
            match current_token {
                TokenType::Message => {
                    let command = self.parse_message()?;
                    commands.push(command);
                },
                TokenType::Publisher => {
                    let command = self.parse_publisher()?;
                    commands.push(command);
                },
                TokenType::Subscribe => {
                    let command = self.parse_subscriber()?;
                    commands.push(command);
                },
                _ => {
                    return Err(ParseError::new(format!(
                        "miss expression, expression cannot start with {current_token:?}, only start with 'message', 'publisher' or 'subscribe",
                    )))
                }
            }
        }
        Ok(commands)
    }

    fn parse_message(&mut self) -> Result<Command, ParseError> {
        self.expected_token(TokenType::Text)?;
        let message_text = self.current_tok.value().unwrap();
        self.consume();
        Ok(Command::Message(message_text))
    }

    fn parse_publisher(&mut self) -> Result<Command, ParseError> {
        self.expected_token(TokenType::Name)?;
        let queue_name = self.current_tok.value().unwrap();
        self.consume();
        Ok(Command::Publisher(queue_name))
    }

    fn parse_subscriber(&mut self) -> Result<Command, ParseError> {
        self.expected_token(TokenType::Name)?;
        let queue_name = self.current_tok.value().unwrap();
        self.expected_token(TokenType::With)?;
        self.expected_token(TokenType::Group)?;
        self.expected_token(TokenType::Name)?;
        let group_name = self.current_tok.value().unwrap();
        self.consume();
        Ok(Command::Subscriber(queue_name, group_name))
    }

    fn expected_token(&mut self, token_type: TokenType) -> Result<(), ParseError> {
        let next_tok_typen = self.next_tok.token_type();
        if next_tok_typen == token_type {
            self.consume();
            return Ok(());
        }
        return Err(ParseError::new(format!(
            "expected {:?} but got {:?}",
            token_type, next_tok_typen
        )));
    }

    fn consume(&mut self) {
        let next_tok = self.lexer.next_token();
        std::mem::swap(&mut self.current_tok, &mut self.next_tok);
        self.next_tok = next_tok;
    }

    fn current_token_is(&self, current_token: TokenType) -> bool {
        self.current_tok.token_type() == current_token
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn given_valid_expresion_should_be_return_command() {
        let cases = [
            (
                "subscribe foo with group bar",
                Command::Subscriber("foo".into(), "bar".into()),
            ),
            (
                "subscribe foo with group bar;",
                Command::Subscriber("foo".into(), "bar".into()),
            ),
            ("publisher foo", Command::Publisher("foo".into())),
            ("publisher foo;", Command::Publisher("foo".into())),
            ("message \"baz\"", Command::Message("baz".into())),
            ("message \"baz\";", Command::Message("baz".into())),
        ];
        for (input, expected) in cases {
            let mut parser = build_parser(input.into());
            let parsed = parser.parse_commands();
            match parsed {
                Ok(commands) => {
                    let command = &commands[0];
                    assert_eq!(
                        &expected, command,
                        "expected {expected:?} but got {command:?}",
                    );
                }
                Err(e) => assert!(
                    false,
                    "fail to parse the command, expected {expected:?}, but got error {e}",
                ),
            }
        }
    }

    #[test]
    fn given_mutli_valid_expresion_should_be_return_command() {
        let cases = [
            (
                "subscribe foo with group bar",
                Command::Subscriber("foo".into(), "bar".into()),
            ),
            ("publisher foo; message \"baz\";", Command::Publisher("foo".into())),
            ("message \"baz\"", Command::Message("baz".into())),
        ];
        for (input, expected) in cases {
            let mut parser = build_parser(input.into());
            let parsed = parser.parse_commands();
            match parsed {
                Ok(commands) => {
                    let command = &commands[0];
                    assert_eq!(
                        &expected, command,
                        "expected {expected:?} but got {command:?}",
                    );
                }
                Err(e) => assert!(
                    false,
                    "fail to parse the command, expected {expected:?}, but got error {e}",
                ),
            }
        }
    }

    fn build_parser(input: String) -> Parser {
        let lexer = Lexer::new(input);
        Parser::new(lexer)
    }
}
