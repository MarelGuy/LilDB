use core::fmt;
use std::fmt::{Display, Formatter};

use logos::Logos;

#[derive(Logos, Debug, PartialEq, Copy, Clone)]
pub enum TokenType<'a> {
    // Database management
    #[token("create")]
    Create,

    #[token("use")]
    Use,

    #[token("drop")]
    Drop,

    #[token("collection")]
    Collection,

    #[token("db")]
    Db,

    #[token("dbs")]
    Dbs,

    #[regex("[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier(&'a str),

    // Document manipulation
    #[token("insert")]
    Insert,

    #[token("update")]
    Update,

    #[token("delete")]
    Delete,

    #[token("find")]
    Find,

    #[token("help")]
    Help,

    #[token("list")]
    List,

    #[token("show")]
    Show,

    // Misc
    #[token("\n")]
    LineFeed,

    #[token(" ")]
    Space,

    #[token("\t")]
    Tab,

    #[token("\0")]
    Null,
}

impl Default for TokenType<'_> {
    fn default() -> Self {
        Self::Null
    }
}

impl Display for TokenType<'_> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub struct Token<'a> {
    pub line: usize,
    pub column: usize,
    pub tok_type: TokenType<'a>,
    pub slice: &'a str,
    pub span: Span,
}

impl<'a> Token<'a> {
    pub fn new(
        line: usize,
        column: usize,
        tok_type: TokenType<'a>,
        slice: &'a str,
        span: Span,
    ) -> Self {
        Self {
            line,
            column,
            tok_type,
            slice,
            span,
        }
    }
}
