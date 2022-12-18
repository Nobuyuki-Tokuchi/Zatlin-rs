use std::fmt::Display;

#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    Unknown(String),
    Minus,
    Or,
    Equal,
    Circumflex,
    Percent,
    Semicolon,
    Value(String),
    Count(usize),
    Variable(String),
    NewLine,
    LeftCirc,
    RightCirc,
    Colon,
    LeftArrow,
    Comma,
}

impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::Unknown(token) => write!(f, "{}", token),
            Self::Minus => write!(f, "-"),
            Self::Or => write!(f, "|"),
            Self::Equal => write!(f, "="),
            Self::Circumflex => write!(f, "^"),
            Self::Percent => write!(f, "%"),
            Self::Semicolon => write!(f, ";"),
            Self::Value(token) => write!(f, "\"{}\"", token),
            Self::Count(token) => write!(f, "{}", token),
            Self::Variable(token) => write!(f, "{}", token),
            Self::NewLine => write!(f, "(NewLine)"),
            Self::LeftCirc => write!(f, "("),
            Self::RightCirc => write!(f, ")"),
            Self::Colon => write!(f, ":"),
            Self::LeftArrow => write!(f, "<-"),
            Self::Comma => write!(f, ","),
        }
    }
}

#[derive(PartialEq, Copy, Clone)]
enum TokenizeMode {
    Normal,
    String,
    Comment,
}

pub fn lexer(text: &str) -> Vec<TokenType> {
    let text = text.chars();

    let mut tokens: Vec<TokenType> = vec![];
    let mut buffer: Vec<char> = vec![];
    let mut mode = TokenizeMode::Normal;

    for c in text {
        if mode == TokenizeMode::Comment {
            if c == '\r' || c == '\n' {
                mode = TokenizeMode::Normal;
            }
        } else if c.is_ascii_whitespace() {
            if mode == TokenizeMode::String {
                buffer.push(c);
            } else {
                if !buffer.is_empty() {
                    let token = String::from_iter(buffer.iter());
                    tokens.push(get_value(&token));
                    buffer.clear();
                }

                if c == '\r' || c == '\n' {
                    tokens.push(TokenType::NewLine);
                }
            }
        } else {
            match c {
                '|' | ';' | '%' | '^' | '=' | '(' | ')' | ':' | ',' => {
                    if !buffer.is_empty() {
                        let token = String::from_iter(buffer.iter());
                        tokens.push(get_value(&token));
                        buffer.clear();
                    }

                    tokens.push(get_tokentype(c));
                },
                '#' => {
                    if !buffer.is_empty() {
                        let token = String::from_iter(buffer.iter());
                        tokens.push(get_value(&token));
                        buffer.clear();
                    }

                    mode = TokenizeMode::Comment;
                },
                '"' => {
                    if mode == TokenizeMode::String {
                        mode = TokenizeMode::Normal;
                        buffer.push(c);

                        let token = String::from_iter(buffer.iter());
                        tokens.push(get_value(&token));
                        buffer.clear();
                    } else {
                        mode = TokenizeMode::String;
                        buffer.push(c);
                    }
                },
                '-' => {
                    let token = String::from_iter(buffer.iter());
                    if token == "<" {
                        tokens.push(TokenType::LeftArrow);
                        buffer.clear();
                    } else {
                        if !buffer.is_empty() {
                            tokens.push(get_value(&token));
                            buffer.clear();
                        }

                        tokens.push(get_tokentype(c));
                    }
                },
                _ => {
                    buffer.push(c);
                }
            }
        }
    }

    tokens
}

pub fn lexer_by_vec(values: Vec<&str>) -> Vec<TokenType> {
    let mut tokens: Vec<TokenType> = vec![];

    for value in values {
        match value {
            "-" => tokens.push(TokenType::Minus),
            "|" => tokens.push(TokenType::Or),
            "%" => tokens.push(TokenType::Percent),
            "^" => tokens.push(TokenType::Circumflex),
            "=" => tokens.push(TokenType::Equal),
            ";" => tokens.push(TokenType::Semicolon),
            "(" => tokens.push(TokenType::LeftCirc),
            ")" => tokens.push(TokenType::RightCirc),
            ":" => tokens.push(TokenType::Colon),
            "," => tokens.push(TokenType::Comma),
            "<-" => tokens.push(TokenType::LeftArrow),
            _ => {
                if value.starts_with("\"") && value.ends_with("\"") {
                    let len = value.len();
                    let token = if len > 2 {
                        TokenType::Value(value[1..(value.len() - 1)].to_string())
                    } else if len == 2 {
                        TokenType::Value("".to_string())
                    } else {
                        TokenType::Unknown(value.to_string())
                    };
                    tokens.push(token);
                } else if let Ok(count) = value.parse() {
                    tokens.push(TokenType::Count(count));
                } else {
                    tokens.push(TokenType::Variable(String::from(value)));
                }
            }
        }
    }

    tokens
}

fn get_value(value: &str) -> TokenType {
    if let Ok(num) = value.parse() {
        TokenType::Count(num)
    } else if value.starts_with('"') && value.ends_with('"') {
        let len = value.len();
        if len > 2 {
            TokenType::Value(value[1..(value.len() - 1)].to_string())
        } else if len == 2 {
            TokenType::Value("".to_string())
        } else {
            TokenType::Unknown(value.to_string())
        }
    } else {
        TokenType::Variable(String::from(value))
    }
}

fn get_tokentype(value: char) -> TokenType {
    match value {
        '-' => TokenType::Minus,
        '|' => TokenType::Or,
        ';' => TokenType::Semicolon,
        '%' => TokenType::Percent,
        '^' => TokenType::Circumflex,
        '=' => TokenType::Equal,
        '(' => TokenType::LeftCirc,
        ')' => TokenType::RightCirc,
        ':' => TokenType::Colon,
        ',' => TokenType::Comma,
        _ => TokenType::Unknown(String::from(value)),
    }
}
