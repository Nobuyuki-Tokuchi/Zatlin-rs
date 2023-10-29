use std::{fs::File, io::Read};

use crate::error::Error;
use crate::lexer::{lexer, lexer_by_vec};
use crate::parser::{parse, Statement};

#[derive(Debug, Clone)]
pub struct Data {
    statements: Vec<Statement>,
} 

impl Data {
    pub(crate) fn get_statements_ref(&self) -> Result<&Vec<Statement>, Error> {
        Ok(self.statements.as_ref())
    }

    pub fn read_file<P>(filename: P) -> Result<Self, Error>
    where
        P: AsRef<std::path::Path>
    {
        let text = {
            let mut f = File::open(filename).map_err(|_| Error::ErrorMessage(String::from("file not found"), None))?;
            let mut contents = String::new();
            f.read_to_string(&mut contents).map_err(|x| Error::ErrorMessage(x.to_string(), None))?;
            contents
        };

        Self::try_from(text)
    }
}

impl TryFrom<Vec<&str>> for Data {
    type Error = Error;

    fn try_from(value: Vec<&str>) -> Result<Self, Self::Error> {
        let tokens = lexer_by_vec(value);
        parse(&tokens).map(|statements| Self {
            statements,
        })
    }
}

impl TryFrom<&str> for Data {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let tokens = lexer(value);
        parse(&tokens).map(|statements| Self {
            statements,
        })
    }
}

impl TryFrom<String> for Data {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let tokens = lexer(&value);
        parse(&tokens).map(|statements| Self {
            statements,
        })
    }
}

impl TryFrom<&String> for Data {
    type Error = Error;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        let tokens = lexer(value);
        parse(&tokens).map(|statements| Self {
            statements,
        })
    }
}

impl std::str::FromStr for Data {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
    }
}
