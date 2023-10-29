
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    InvalidToken(String, String, usize),
    EndOfToken(String, usize),
    UnknownToken(String, usize),
    NotFoundPattern,
    NotFoundVariable(String),
    OverRetryCount,
    ErrorMessage(String, Option<usize>),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::InvalidToken(parse_point, token, index) => write!(f, "Invalid token in {} : {}, index: {}", parse_point, token, index),
            Self::EndOfToken(parse_point, index ) => write!(f, "End of token in {} : index: {}", parse_point, index),
            Self::UnknownToken(token, index) => write!(f, "Unknown token : {}, index: {}", token, index),
            Self::NotFoundPattern => write!(f, "Not found patterns."),
            Self::NotFoundVariable(key) => write!(f, "Not found variable: {}", key),
            Self::OverRetryCount => write!(f, "Retry count is over limit."),
            Self::ErrorMessage(message, index) => {
                match index {
                    Some(index) => write!(f, "{}: index: {}", message, index),
                    None => write!(f, "{}", message),
                }
            }
        }
    }
}
