pub mod lexer;
pub mod parser;
pub mod error;

use error::ErrorValue;
use lexer::{lexer, lexer_by_vec};
use parser::{parse, Statement};

#[derive(Debug, Clone)]
pub struct ZatlinData {
    statements: Vec<Statement>,
} 

impl ZatlinData {
    pub fn get_statements_ref(&self) -> &Vec<Statement> {
        self.statements.as_ref()
    }
}

impl TryFrom<Vec<&str>> for ZatlinData {
    type Error = ErrorValue;

    fn try_from(value: Vec<&str>) -> Result<Self, Self::Error> {
        let tokens = lexer_by_vec(value);
        parse(&tokens).map(|x| Self {
            statements: x
        })
    }
}

impl TryFrom<&str> for ZatlinData {
    type Error = ErrorValue;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let tokens = lexer(value);
        parse(&tokens).map(|x| Self {
            statements: x
        })
    }
}