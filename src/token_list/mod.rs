use crate::lexer::token::{Span, Token, TokenType};

#[derive(PartialEq, Debug, Clone)]
pub struct TokenList<'a> {
    pub tokens: Vec<Token<'a>>,
    pub current_token: Token<'a>,
    current_index: usize,
}

impl<'a> TokenList<'a> {
    pub fn new(tokens: Vec<Token<'a>>) -> Self {
        Self {
            tokens,
            current_token: Token::new(0, 0, TokenType::Null, "", Span { end: 0, start: 0 }),
            current_index: 0,
        }
    }

    pub fn next(&mut self, steps: usize) {
        self.current_index += steps;

        if self.current_index < self.tokens.len() {
            self.current_token = self.tokens[self.current_index];
        }
    }
}
