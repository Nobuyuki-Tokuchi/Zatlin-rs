pub mod lexer;
pub mod parser;
pub mod error;

use std::{fs::File, io::Read};

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

    pub fn read_file<P>(filename: P) -> Result<Self, ErrorValue>
    where
        P: AsRef<std::path::Path>
    {
        let text = {
            let mut f = File::open(filename).map_err(|_| ErrorValue::ErrorMessage(String::from("file not found"), None))?;
            let mut contents = String::new();
            f.read_to_string(&mut contents).map_err(|x| ErrorValue::ErrorMessage(x.to_string(), None))?;
            contents
        };

        Self::try_from(text)
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

impl TryFrom<String> for ZatlinData {
    type Error = ErrorValue;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let tokens = lexer(&value);
        parse(&tokens).map(|x| Self {
            statements: x
        })
    }
}

impl TryFrom<&String> for ZatlinData {
    type Error = ErrorValue;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        let tokens = lexer(value);
        parse(&tokens).map(|x| Self {
            statements: x
        })
    }
}

impl std::str::FromStr for ZatlinData {
    type Err = ErrorValue;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
    }
}
