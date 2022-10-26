pub mod lexer;
pub mod parser;
pub mod error;

use error::ErrorValue;
use lexer::lexer;
use parser::{parse, Statement};

#[derive(Debug, Clone)]
pub struct ZatlinData {
    statements: Vec<Statement>,
} 

impl ZatlinData {
    pub fn new(statements: Vec<Statement>) -> Self {
        Self { statements }
    }

    pub fn new_str(text: &str) -> Result<Self, ErrorValue> {
        let tokens = lexer(text);
        parse(&tokens).map(|x| Self {
            statements: x
        })
    }

    pub fn get_statements_ref(&self) -> &Vec<Statement> {
        self.statements.as_ref()
    }
}