

use crate::lexer::{TokenType};

#[derive(Debug, Clone)]
pub enum Statement {
    Define(String, Expression),
    Generate(Expression)
}

#[derive(Debug, Clone)]
pub struct Expression {
    pub patterns: Vec<Pattern>,
    pub excludes: Vec<ExcludePattern>,
}

#[derive(Debug, Clone)]
pub struct Pattern {
    pub values: Vec<Value>,
    pub count: usize,
}

#[derive(Debug, Clone)]
pub struct ExcludePattern {
    pub values: Vec<Value>,
    pub mode: ExtractMode,
}

#[derive(Debug, Clone)]
pub enum ExtractMode {
    None,
    Forward,
    Backward,
    Exact,
}

#[derive(Debug, Clone)]
pub struct Value {
    pub value: String,
    pub is_variable: bool,
}

impl Value {
    fn literal(value: &str) -> Self {
        Value {
            value: String::from(value),
            is_variable: false,
        }
    }

    fn variable(value: &str) -> Self {
        Value {
            value: String::from(value),
            is_variable: true,
        }
    }
}

pub fn parse(tokens: &Vec<TokenType>) -> Result<Vec<Statement>, String> {
    let mut statements = vec![];
    
    let mut index = 0;
    let length = tokens.len();
    while index < length {
        if let Some(value) = tokens.get(index) {
            match value {
                TokenType::Variable(value) => {
                    let (define, next_index) = parse_define(&value, &tokens, index + 1)?;
                    statements.push(define);
                    index = next_index;
                },
                TokenType::Percent => {
                    let (generate, next_index) = parse_generate(&tokens, index + 1)?;
                    statements.push(generate);
                    index = next_index;
                },
                TokenType::Unknown(value) => {
                    return Err(format!("Unknown token: {}, index: {}", value, index))
                },
                TokenType::NewLine => {
                    index = index + 1
                }
                _ => {
                    return Err(format!("Invalid token: {:?}, index: {}", value, index))
                }
            }
        } else {
            return Err(format!("End of token in statement. length: {}, index: {}", length, index))
        }
    }
    
    Ok(statements)
}


fn parse_define(value: &str, tokens: &[TokenType], index: usize) -> Result<(Statement, usize), String> {
    let next_index = if let Some(next) = tokens.get(index) {
        if &TokenType::Equal == next {
            index + 1
        } else {
            return Err(format!("Invalid token in define variable: {:?}, index: {}", value, index))
        }
    } else {
        return Err(format!("End of token in define variable. index: {}", index))
    };

    let (expr, next_index) = parse_expression(&tokens, next_index)?;

    let next_index = if let Some(value) = tokens.get(next_index) {
        if &TokenType::Semicolon == value || &TokenType::NewLine == value {
            next_index + 1
        } else {
            return Err(format!("Invalid token in define variable: {:?}, index: {}", value, index))
        }
    } else {
        return Err(format!("End of token in define variable. index: {}", next_index))
    };

    Ok((Statement::Define(value.to_string(), expr), next_index))
}

fn parse_generate(tokens: &[TokenType], index: usize) -> Result<(Statement, usize), String> {
    let (expr, next_index) = parse_expression(&tokens, index)?;

    let next_index = if let Some(value) = tokens.get(next_index) {
        if &TokenType::Semicolon == value {
            next_index + 1
        } else {
            return Err(format!("Invalid token in generate: {:?}, index: {}", value, index))
        }
    } else {
        return Err(format!("End of token in generate. index: {}", next_index))
    };

    Ok((Statement::Generate(expr), next_index))
}

fn parse_expression(tokens: &[TokenType], index: usize) -> Result<(Expression, usize), String> {
    let (patterns, next_index) = parse_patterns(&tokens, index)?;

    let (excludes, next_index) = if let Some(TokenType::Minus) = tokens.get(next_index) {
        parse_exclude_patterns(&tokens, next_index + 1)?
    } else {
        (Vec::new(), next_index)
    };

    Ok((Expression { patterns: patterns, excludes: excludes }, next_index))
}

fn parse_patterns(tokens: &[TokenType], index: usize) -> Result<(Vec<Pattern>, usize), String> {
    let (pattern, mut next_index) = parse_pattern(&tokens, index)?;
    let mut patterns = vec![pattern];

    loop {
        if let Some(value) = tokens.get(next_index) {
            if &TokenType::Or == value {
                next_index = next_index + 1;
            } else {
                break;
            }
        } else {
            return Err(format!("End of token in patterns. index: {}", next_index))
        };

        if let Ok((pattern, index)) = parse_pattern(&tokens, next_index) {
            patterns.push(pattern);
            next_index = index;
        } else {
            return Err(format!("Next pattern is nothing in patterns. index: {}", next_index))
        }
    }

    Ok((patterns, next_index))
}

fn parse_pattern(tokens: &[TokenType], index: usize) -> Result<(Pattern, usize), String> {
    let (values, next_index) = parse_values(&tokens, index)?;
    let (count, next_index) = match tokens.get(next_index) {
        Some(value) => {
            if let TokenType::Count(value) = value {
                (*value, next_index + 1)
            } else {
                (1, next_index)
            }
        },
        None => {
            return Err(format!("End of token in pattern. index: {}", index))
        }
    };

    Ok((Pattern { values: values, count: count }, next_index))
}

fn parse_exclude_patterns(tokens: &[TokenType], index: usize) -> Result<(Vec<ExcludePattern>, usize), String> {
    let (pattern, mut next_index) = parse_exclude_pattern(&tokens, index)?;
    let mut patterns = vec![pattern];

    loop {
        if let Some(value) = tokens.get(next_index) {
            if &TokenType::Or == value {
                next_index = next_index + 1;
            } else {
                break;
            }
        } else {
            return Err(format!("End of token in exclude patterns. index: {}", next_index))
        }

        if let Ok((pattern, index)) = parse_exclude_pattern(&tokens, next_index) {
            patterns.push(pattern);
            next_index = index;
        } else {
            break;
        }
    }

    Ok((patterns, next_index))
}

fn parse_exclude_pattern(tokens: &[TokenType], index: usize) -> Result<(ExcludePattern, usize), String> {
    let (is_prefix, next_index) = if let Some(token) = tokens.get(index) {
        match token {
            TokenType::Circumflex => (true, index + 1),
            _ => (false, index),
        }
    } else {
        return Err(format!("End of token in exclude pattern (prefix). index: {}", index))
    };
    
    let (values, next_index) = parse_values(&tokens, next_index)?;
    if values.iter().any(|x| x.is_variable) {
        return Err(String::from("Exclude Pattern can't include variable."))
    }
    
    let (is_postfix, next_index) = if let Some(token) = tokens.get(next_index) {
        match token {
            TokenType::Circumflex => (true, next_index + 1),
            _ => (false, next_index),
        }
    } else {
        (false, next_index)
    };

    let mode = match (is_prefix, is_postfix) {
        (false, false) => ExtractMode::None,
        (false, true) => ExtractMode::Backward,
        (true, false) => ExtractMode::Forward,
        (true, true) => ExtractMode::Exact,
    };

    Ok((ExcludePattern { values: values, mode: mode }, next_index))
}

fn parse_values(tokens: &[TokenType], index: usize) -> Result<(Vec<Value>, usize), String> {
    let (value, mut next_index) = parse_value(&tokens, index)?;
    let mut values = vec![value];

    loop {
        if let Ok((value, index)) = parse_value(&tokens, next_index) {
            values.push(value);
            next_index = index;
        } else {
            break;
        }
    }

    Ok((values, next_index))
}

fn parse_value(tokens: &[TokenType], index: usize) -> Result<(Value, usize), String> {
    if let Some(token) = tokens.get(index) {
        match token {
            TokenType::Value(value) => Ok((Value::literal(&value), index + 1)),
            TokenType::Variable(value) => Ok((Value::variable(&value), index + 1)),
            _ => Err(format!("Invalid token in value: {:?}, index: {}", token, index)),
        }
    } else {
        Err(format!("End of token in value. index: {}", index))
    }
}


#[cfg(test)]
mod parse_test {
    use super::Statement;

    fn execute(s: &str) -> Result<Vec<Statement>, String> {
        let tokens = crate::lexer::lexer(s);
        crate::parser::parse(&tokens)
    }
    
    #[test]
    fn default() {
        let result = execute(r#"
        Cs = "" | "b" | "p" | "f" | "v" | "d" | "t" | "s" | "z" | "c" | "j" | "g" | "k" | "h" | "q" | "r" | "w" | "n" | "m"
        Ce = "" | "b" | "d" | "g" | "m" | "n" | "h"

        Va = "a" | "á" | "à" | "ä"
        Ve = "e" | "é" | "è" | "ë"
        Vi = "i" | "í" | "ì" | "ï"
        Vo = "o" | "ó" | "ò" | "ö"
        Vu = "u" | "ú" | "ù" | "ü"
        Vy = "y" | "ý" | "ỳ" | "ÿ"

        Vxi = Va "i" | Ve "i" | Vo "i" | Vi "a" | Vi "e"
        Vxu = Va "u" | Vo "u" | Vu "e" | Vu "i"
        Vx = Va | Ve | Vi | Vo | Vu | Vy | Vxi | Vxu

        % Cs Vx Ce | Cs Vx Ce Cs Vx Ce - ^ "y" | ^ "ý" | ^ "ỳ" | ^ "ÿ" | ^ "wu" | ^ "wú" | ^ "wù" | ^ "wü" | ^ "hy" | ^ "hý" | ^ "hỳ" | ^ "hÿ" | ^ "qy" | ^ "qý" | ^ "qỳ" | ^ "qÿ" | ^ "ry" | ^ "rý" | ^ "rỳ" | ^ "rÿ" | ^ "ny" | ^ "ný" | ^ "nỳ" | ^ "nÿ" | ^ "my" | ^ "mý" | ^ "mỳ" | ^ "mÿ";
        "#);

        println!("{:?}", result);
        assert!(result.is_ok())
    }

    #[test]
    fn use_semicolon() {
        let result = execute(r#"
        Cs = "" | "b" | "p" | "f" | "v" | "d" | "t" | "s" | "z" | "c" | "j" | "g" | "k" | "h" | "q" | "r" | "w" | "n" | "m";
        Ce = "" | "b" | "d" | "g" | "m" | "n" | "h";

        Va = "a" | "á" | "à" | "ä";
        Ve = "e" | "é" | "è" | "ë";
        Vi = "i" | "í" | "ì" | "ï";
        Vo = "o" | "ó" | "ò" | "ö";
        Vu = "u" | "ú" | "ù" | "ü";
        Vy = "y" | "ý" | "ỳ" | "ÿ";

        Vxi = Va "i" | Ve "i" | Vo "i" | Vi "a" | Vi "e";
        Vxu = Va "u" | Vo "u" | Vu "e" | Vu "i";
        Vx = Va | Ve | Vi | Vo | Vu | Vy | Vxi | Vxu;

        % Cs Vx Ce | Cs Vx Ce Cs Vx Ce - ^ "y" | ^ "ý" | ^ "ỳ" | ^ "ÿ" | ^ "wu" | ^ "wú" | ^ "wù" | ^ "wü" | ^ "hy" | ^ "hý" | ^ "hỳ" | ^ "hÿ" | ^ "qy" | ^ "qý" | ^ "qỳ" | ^ "qÿ" | ^ "ry" | ^ "rý" | ^ "rỳ" | ^ "rÿ" | ^ "ny" | ^ "ný" | ^ "nỳ" | ^ "nÿ" | ^ "my" | ^ "mý" | ^ "mỳ" | ^ "mÿ";
        "#);

        println!("{:?}", result);
        assert!(result.is_ok())
    }

    #[test]
    fn multiple_define_in_line() {
        let result = execute(r#"
        C = "p" | "f" | "t" | "s" | "k" | "h"; V = "a" | "i" | "u";

        % C V | C V C | V C | V C V;
        "#);

        println!("{:?}", result);
        assert!(result.is_ok())
    }

    #[test]
    fn nothing_semicolon() {
        let result = execute(r#"
        C = "p" | "f" | "t" | "s" | "k" | "h"
        V = "a" | "i" | "u"

        # invalid statement in generate.
        # semicolon is nothing.
        % C V | C V C | V C | V C V
        "#);

        println!("{:?}", result);
        assert!(result.is_err());
        assert!(result.unwrap_err().starts_with("Invalid token in generate: NewLine"))
    }

    #[test]
    fn invalid_define_variable() {
        let result = execute(r#"
        C = "p" | "f" | "t" |
            "s" | "k" | "h";
        V = "a" | "i" | "u";

        % C V | C V C | V C | V C V;
        "#);

        println!("{:?}", result);
        assert!(result.is_err());
        assert!(result.unwrap_err().starts_with("Next pattern is nothing in patterns."))
    }
}