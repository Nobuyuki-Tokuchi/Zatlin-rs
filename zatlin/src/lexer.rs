use std::fmt::Display;


#[derive(Debug, PartialEq, Clone)]
pub(crate) struct Token {
    pub row: u64,
    pub column: u64,
    pub tokentype: TokenType,
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum TokenType {
    Unknown(String),
    Minus,
    Or,
    Equal,
    Circumflex,
    Percent,
    Semicolon,
    Value(String),
    Count(f64),
    Variable(String),
    NewLine,
    LeftCirc,
    RightCirc,
    Ampersand(u32),
}

impl Token {
    pub fn new(row: u64, column: u64, tokentype: TokenType) -> Self {
        Token {
            row,
            column,
            tokentype
        }
    }

    pub fn newline(row: u64, column: u64) -> Self {
        Self::new(row, column, TokenType::NewLine)
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ row: {}, column: {}, token: {} }}", self.row, self.column, self.tokentype)
    }
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
            Self::Ampersand(index) => write!(f, "{}", index + 1),
        }
    }
}

#[derive(PartialEq, Copy, Clone)]
enum TokenizeMode {
    Normal,
    String,
    Comment,
}

pub(crate) fn lexer(text: &str) -> Vec<Token> {
    let text = text.chars();

    let mut tokens: Vec<Token> = vec![];
    let mut buffer: Vec<char> = vec![];
    let mut mode = TokenizeMode::Normal;
    let mut row: u64 = 1;
    let mut column: u64 = 0;
    let mut length: u64 = 0;

    for c in text {
        match mode {
            TokenizeMode::String => {
                if c == '"' {
                    mode = TokenizeMode::Normal;
                    buffer.push(c);
                    length += 1;
    
                    let token = String::from_iter(buffer.iter());
                    tokens.push(get_value(row, column, &token));
                    buffer.clear();
    
                    column += length;
                    length = 0;
                } else if c == '\r' || c == '\n' {
                    mode = TokenizeMode::Normal;
    
                    let token = String::from_iter(buffer.iter());
                    tokens.push(get_value(row, column, &token));
                    buffer.clear();
    
                    tokens.push(Token::newline(row, column));
                    row += 1;
                    column = 0;
                } else {
                    buffer.push(c);
                    length += 1;
                }
            },
            TokenizeMode::Comment => {
                if c == '\r' || c == '\n' {
                    mode = TokenizeMode::Normal;
                    row += 1;
                    column = 0;
                    length = 0;
                }
            },
            TokenizeMode::Normal => {
                if c.is_ascii_whitespace() {
                    if !buffer.is_empty() {
                        let token = String::from_iter(buffer.iter());
                        tokens.push(get_value(row, column, &token));
                        buffer.clear();

                        column += length;
                    }

                    if c == '\r' || c == '\n' {
                        tokens.push(Token::newline(row, column));
                        row += 1;
                        column = 0;
                    } else {
                        column += 1;
                    }
                    length = 0;
                } else {
                    match c {
                        '-' | '|' | ';' | '%' | '^' | '=' | '(' | ')' => {
                            if !buffer.is_empty() {
                                let token = String::from_iter(buffer.iter());
                                tokens.push(get_value(row, column, &token));
                                buffer.clear();
            
                                column += length;
                                length = 0;
                                
                            }
    
                            tokens.push(get_token(row, column, c));
                            column += 1;
                        },
                        '#' => {
                            if !buffer.is_empty() {
                                let token = String::from_iter(buffer.iter());
                                tokens.push(get_value(row, column, &token));
                                buffer.clear();
                            }
    
                            mode = TokenizeMode::Comment;
                            column = 0;
                            length = 0;
                        },
                        '"' => {
                            if !buffer.is_empty() {
                                let token = String::from_iter(buffer.iter());
                                tokens.push(get_value(row, column, &token));
                                buffer.clear();
            
                                column += length;
                                length = 0;
                            }
    
                            mode = TokenizeMode::String;
                            buffer.push(c);
    
                            length += 1;
                        
                        },
                        '&' => {
                            if !buffer.is_empty() {
                                let token = String::from_iter(buffer.iter());
                                tokens.push(get_value(row, column, &token));
                                buffer.clear();
            
                                column += length;
                                length = 0;
                            }
    
                            buffer.push(c);
                            length += 1;
                        }
                        _ => {
                            buffer.push(c);
                            length += 1;
                        }
                    }
                }
            },
        }
    }

    if !buffer.is_empty() {
        let token = String::from_iter(buffer.iter());
        let last_token = get_value(row, column, &token);
        tokens.push(last_token);
    }

    tokens
}

pub(crate) fn lexer_by_vec(values: Vec<&str>) -> Vec<Token> {
    let mut tokens: Vec<Token> = vec![];

    for value in values {
        match value {
            "-" => tokens.push(Token::new(1, 0, TokenType::Minus)),
            "|" => tokens.push(Token::new(1, 0, TokenType::Or)),
            "%" => tokens.push(Token::new(1, 0, TokenType::Percent)),
            "^" => tokens.push(Token::new(1, 0, TokenType::Circumflex)),
            "=" => tokens.push(Token::new(1, 0, TokenType::Equal)),
            ";" => tokens.push(Token::new(1, 0, TokenType::Semicolon)),
            "(" => tokens.push(Token::new(1, 0, TokenType::LeftCirc)),
            ")" => tokens.push(Token::new(1, 0, TokenType::RightCirc)),
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
                    tokens.push(Token::new(1, 0, token));
                } else if let Ok(count) = value.parse() {
                    tokens.push(Token::new(1, 0, TokenType::Count(count)));
                } else {
                    tokens.push(Token::new(1, 0, TokenType::Variable(String::from(value))));
                }
            }
        }
    }

    tokens
}

fn get_value(row: u64, column: u64, value: &str) -> Token {
    let tokentype = if let Ok(num) = value.parse() {
        TokenType::Count(num)
    } else if value.starts_with('"') {
        if value.ends_with('"') {
            let len = value.len();
            if len > 2 {
                TokenType::Value(value[1..(value.len() - 1)].to_string())
            } else if len == 2 {
                TokenType::Value("".to_string())
            } else {
                TokenType::Unknown(value.to_string())
            }
        } else {
            TokenType::Unknown(value.to_string())
        }
    } else if let Some(index) = value.strip_prefix("&").and_then(|x| x.parse::<u32>().ok()) {
        TokenType::Ampersand(index - 1)
    } else {
        TokenType::Variable(String::from(value))
    };

    Token::new(row, column, tokentype)
}

fn get_token(row: u64, column: u64, value: char) -> Token {
    let tokentype = match value {
        '-' => TokenType::Minus,
        '|' => TokenType::Or,
        ';' => TokenType::Semicolon,
        '%' => TokenType::Percent,
        '^' => TokenType::Circumflex,
        '=' => TokenType::Equal,
        '(' => TokenType::LeftCirc,
        ')' => TokenType::RightCirc,
        _ => TokenType::Unknown(String::from(value)),
    };

    Token::new(row, column, tokentype)
}


#[cfg(test)]
mod lexer_test {
    use super::{TokenType, Token};

    fn execute(s: &str) -> Vec<Token> {
        crate::lexer::lexer(&s)
    }

    #[test]
    fn simple1() {
        let result = execute(r#"identifier = "foo" | "bar" | "baz" "two" "three""#);

        println!("{:?}", result);
        
        let unknown_tokens: Vec<(usize, &Token)> = result.iter().enumerate().filter(|(_, x)| {
            match x.tokentype {
                TokenType::Unknown(_) => true,
                _ => false
            }
        }).collect();

        println!("{:?}", unknown_tokens);
        assert!(unknown_tokens.is_empty());
    }

    #[test]
    fn simple2() {
        let result = execute(r#"identifier = inner inner | "string" inner | "string" "string" | "string""#);

        println!("{:?}", result);
        
        let unknown_tokens: Vec<(usize, &Token)> = result.iter().enumerate().filter(|(_, x)| {
            match x.tokentype {
                TokenType::Unknown(_) => true,
                _ => false
            }
        }).collect();

        println!("{:?}", unknown_tokens);
        assert!(unknown_tokens.is_empty());
    }

    #[test]
    fn with_weight() {
        let result = execute(r#"identifier = "foo" 3 | "bar" 1.94 | "baz" .2 | unweighted | "qux" 50."#);

        println!("{:?}", result);
        
        let unknown_tokens: Vec<(usize, &Token)> = result.iter().enumerate().filter(|(_, x)| {
            match x.tokentype {
                TokenType::Unknown(_) => true,
                _ => false
            }
        }).collect();

        println!("{:?}", unknown_tokens);
        assert!(unknown_tokens.is_empty());
    }

    #[test]
    fn with_exclusion1() {
        let result = execute(r#"identifier = "foo" | "bar" | "baz" "qux" - "exclusion" | exclusion"#);

        println!("{:?}", result);
        
        let unknown_tokens: Vec<(usize, &Token)> = result.iter().enumerate().filter(|(_, x)| {
            match x.tokentype {
                TokenType::Unknown(_) => true,
                _ => false
            }
        }).collect();

        println!("{:?}", unknown_tokens);
        assert!(unknown_tokens.is_empty());
    }

    #[test]
    fn with_exclusion2() {
        let result = execute(r#"identifier = "foo" | "bar" | "baz" "qux" - ^ exact ^ | ^ prefix | suffix ^"#);

        println!("{:?}", result);
        
        let unknown_tokens: Vec<(usize, &Token)> = result.iter().enumerate().filter(|(_, x)| {
            match x.tokentype {
                TokenType::Unknown(_) => true,
                _ => false
            }
        }).collect();

        println!("{:?}", unknown_tokens);
        assert!(unknown_tokens.is_empty());
    }

    #[test]
    fn complex1() {
        let result = execute(r#"identifier = ("a" | "b" | "c") ("d" | "e") | "f" ("g" | "h")"#);

        println!("{:?}", result);
        
        let unknown_tokens: Vec<(usize, &Token)> = result.iter().enumerate().filter(|(_, x)| {
            match x.tokentype {
                TokenType::Unknown(_) => true,
                _ => false
            }
        }).collect();

        println!("{:?}", unknown_tokens);
        assert!(unknown_tokens.is_empty());
    }

    #[test]
    fn complex2() {
        let result = execute(r#"identifier = "a" ("b" 3 | "c" | ("d" | "e" "f" 1.7 | inner "g") 5) | ("h" | inner inner 4) "i" 2 - ^ ("j" | "k") "l" inner | "m" "n" ^"#);

        println!("{:?}", result);
        
        let unknown_tokens: Vec<(usize, &Token)> = result.iter().enumerate().filter(|(_, x)| {
            match x.tokentype {
                TokenType::Unknown(_) => true,
                _ => false
            }
        }).collect();

        println!("{:?}", unknown_tokens);
        assert!(unknown_tokens.is_empty());
    }


    #[test]
    fn main_generatable() {
        let result = execute(r#"% "a" ("b" 3 | "c" | ("d" | "e" "f" 1.7 | inner "g") 5) | ("h" | inner inner 4) "i" 2 - ^ ("j" | "k") "l" inner | "m" "n" ^;"#);

        println!("{:?}", result);
        
        let unknown_tokens: Vec<(usize, &Token)> = result.iter().enumerate().filter(|(_, x)| {
            match x.tokentype {
                TokenType::Unknown(_) => true,
                _ => false
            }
        }).collect();

        println!("{:?}", unknown_tokens);
        assert!(unknown_tokens.is_empty());
    }
}