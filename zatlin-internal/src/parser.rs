
use std::rc::Rc;
use crate::lexer::TokenType;
use crate::error::ErrorValue;

#[derive(Debug, Clone)]
pub enum Statement {
    Define(DefineStruct),
    Generate(Rc<Expression>)
}

#[derive(Debug, Clone)]
pub struct DefineStruct {
    pub name: String,
    pub expr: Rc<Expression>
}

#[derive(Debug, Clone)]
pub struct Expression {
    pub patterns: Vec<Pattern>,
    pub excludes: Vec<Pattern>,
}

#[derive(Debug, Clone)]
pub struct Pattern {
    pub values: Vec<Value>,
    pub count: usize,
    pub mode: ExtractMode,
}

#[derive(Debug, Clone)]
pub enum ExtractMode {
    None,
    Forward,
    Backward,
    Exact,
}

impl Pattern {
    fn new(values: Vec<Value>, count: usize) -> Self {
        Self { values, count, mode: ExtractMode::None }
    }

    fn exclude_new(values: Vec<Value>, mode: ExtractMode) -> Self {
        Self { values, count: 0, mode }
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    Literal(String),
    Variable(String),
}

impl Value {
    pub fn is_variable(&self) -> bool {
        if let &Self::Variable(_) = self {
            true
        } else {
            false
        }
    }
}

pub fn parse(tokens: &Vec<TokenType>) -> Result<Vec<Statement>, ErrorValue> {
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
                    return Err(ErrorValue::UnknownToken(value.clone(), index))
                },
                TokenType::NewLine => {
                    index = index + 1
                }
                _ => {
                    return Err(ErrorValue::InvalidToken(String::from("statement"), value.to_string(), index))
                }
            }
        } else {
            return Err(ErrorValue::ErrorMessage(format!("End of token in statement. length: {}, index: {}", length, index), None))
        }
    }
    
    Ok(statements)
}


fn parse_define(value: &str, tokens: &[TokenType], index: usize) -> Result<(Statement, usize), ErrorValue> {
    let next_index = index;
    let next_index = if let Some(next) = tokens.get(next_index) {
        match next {
            &TokenType::Equal => next_index + 1,
            _ => return Err(ErrorValue::InvalidToken(String::from("define variable"), next.to_string(), index))
        }
    } else {
        return Err(ErrorValue::EndOfToken(String::from("define variable"), index))
    };

    let (expr, next_index) = parse_expression(&tokens, next_index)?;

    if let Some(token) = tokens.get(next_index) {
        if &TokenType::Semicolon == token || &TokenType::NewLine == token {
            Ok((Statement::Define(DefineStruct { name: String::from(value), expr: Rc::new(expr) }), next_index + 1))
        } else {
            Err(ErrorValue::InvalidToken(String::from("expression of define variable"), token.to_string(), index))
        }
    } else {
        Err(ErrorValue::EndOfToken(String::from("expression of define variable"), index))
    }
}

fn parse_generate(tokens: &[TokenType], index: usize) -> Result<(Statement, usize), ErrorValue> {
    let (expr, next_index) = parse_expression(&tokens, index)?;

    if let Some(token) = tokens.get(next_index) {
        if &TokenType::Semicolon == token {
            Ok((Statement::Generate(Rc::new(expr)), next_index + 1))
        } else {
            Err(ErrorValue::InvalidToken(String::from("generate"), token.to_string(), next_index))
        }
    } else {
        Err(ErrorValue::EndOfToken(String::from("generate"), next_index))
    }
}

fn parse_expression(tokens: &[TokenType], index: usize) -> Result<(Expression, usize), ErrorValue> {
    let (patterns, next_index) = parse_patterns(&tokens, index)?;

    let (excludes, next_index) = if let Some(TokenType::Minus) = tokens.get(next_index) {
        parse_exclude_patterns(&tokens, next_index + 1)?
    } else {
        (Vec::new(), next_index)
    };
    let excludes = expand_exclude_inner_pattern(excludes);

    Ok((Expression { patterns, excludes }, next_index))
}

fn expand_exclude_inner_pattern(patterns: Vec<Pattern>) -> Vec<Pattern> {
    let mut result_list: Vec<Pattern> = vec![];

    for pattern in patterns.into_iter() {
        let mut value_list: Vec<String> = vec![String::new()];
        for value in pattern.values.into_iter() {
            match value {
                Value::Literal(literal) => {
                    value_list = value_list.into_iter().map(|x| x + &literal).collect();
                },
                Value::Variable(_) => { },
            }
        }
        result_list.push(Pattern::exclude_new(value_list.into_iter().map(|x| Value::Literal(x)).collect(), pattern.mode))
    }

    result_list
}

fn parse_patterns(tokens: &[TokenType], index: usize) -> Result<(Vec<Pattern>, usize), ErrorValue> {
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
            return Err(ErrorValue::EndOfToken(String::from("patterns"), next_index))
        };

        if let Ok((pattern, index)) = parse_pattern(&tokens, next_index) {
            patterns.push(pattern);
            next_index = index;
        } else {
            return Err(ErrorValue::ErrorMessage(String::from("Next pattern is nothing in patterns"), Some(next_index)))
        }
    }

    Ok((patterns, next_index))
}

fn parse_pattern(tokens: &[TokenType], index: usize) -> Result<(Pattern, usize), ErrorValue> {
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
            return Err(ErrorValue::EndOfToken(String::from("pattern"), index))
        }
    };

    Ok((Pattern::new(values, count), next_index))
}

fn parse_values(tokens: &[TokenType], index: usize) -> Result<(Vec<Value>, usize), ErrorValue> {
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

fn parse_value(tokens: &[TokenType], index: usize) -> Result<(Value, usize), ErrorValue> {
    if let Some(token) = tokens.get(index) {
        match token {
            TokenType::Value(value) => Ok((Value::Literal(value.clone()), index + 1)),
            TokenType::Variable(value) => Ok((Value::Variable(value.clone()), index + 1)),
            _ => Err(ErrorValue::InvalidToken(String::from("value"), token.to_string(), index)),
        }
    } else {
            Err(ErrorValue::EndOfToken(String::from("value"), index))
    }
}

fn parse_exclude_patterns(tokens: &[TokenType], index: usize) -> Result<(Vec<Pattern>, usize), ErrorValue> {
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
            return Err(ErrorValue::EndOfToken(String::from("exclude patterns"), next_index))
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

fn parse_exclude_pattern(tokens: &[TokenType], index: usize) -> Result<(Pattern, usize), ErrorValue> {
    let (is_prefix, next_index) = if let Some(token) = tokens.get(index) {
        match token {
            TokenType::Circumflex => (true, index + 1),
            _ => (false, index),
        }
    } else {
        return Err(ErrorValue::EndOfToken(String::from("exclude pattern (prefix)"), index))
    };
    
    let (values, next_index) = parse_exclude_values(&tokens, next_index)?;
    if values.iter().any(|x| x.is_variable()) {
        return Err(ErrorValue::ExcludeIncludeVariable)
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

    Ok((Pattern::exclude_new(values, mode), next_index))
}

fn parse_exclude_values(tokens: &[TokenType], index: usize) -> Result<(Vec<Value>, usize), ErrorValue> {
    let (value, mut next_index) = parse_exclude_value(&tokens, index)?;
    let mut values = vec![value];

    loop {
        if let Ok((value, index)) = parse_exclude_value(&tokens, next_index) {
            values.push(value);
            next_index = index;
        } else {
            break;
        }
    }

    Ok((values, next_index))
}

fn parse_exclude_value(tokens: &[TokenType], index: usize) -> Result<(Value, usize), ErrorValue> {
    if let Some(token) = tokens.get(index) {
        match token {
            TokenType::Value(value) => Ok((Value::Literal(value.clone()), index + 1)),
            _ => Err(ErrorValue::InvalidToken(String::from("exclude value"), token.to_string(), index)),
        }
    } else {
        Err(ErrorValue::EndOfToken(String::from("exclude value"), index))
    }
}

#[cfg(test)]
mod parse_test {
    use crate::{parser::DefineStruct, lexer::TokenType};

    use super::{Statement, ErrorValue};

    fn execute(s: &str) -> Result<Vec<Statement>, ErrorValue> {
        let tokens = crate::lexer::lexer(s);
        crate::parser::parse(&tokens)
    }
    
    #[test]
    fn default() {
        let result = execute(r#"
        # metapi
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
        # metapi
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
    fn expand_exclude_check() {
        let result = execute(r#"
        C = "p" | "f" | "t" | "s" | "k" | "h";
        V = "a" | "i" | "u";

        % C V | C V C | V C | V C V | C C V C | C V C C - "h" "u" | "a" "h" ^ | "i" "h" ^ | "u" "h" ^ | "f" "h" | "s" "h";
        "#);

        println!("{:?}", result);
        assert!(result.is_ok());

        let excludes_check: bool = result.unwrap().iter().fold(true, |acc, x| {
            acc && match x {
                Statement::Define(DefineStruct { name: _, expr }) => {
                    if expr.excludes.is_empty() {
                        true
                    } else {
                        expr.excludes.iter().any(|x| x.values.len() == 1)
                    }
                },
                Statement::Generate(expr) => {
                    if expr.excludes.is_empty() {
                        true
                    } else {
                        expr.excludes.iter().any(|x| x.values.len() == 1)
                    }
                }
            }
        });
        assert!(excludes_check);
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
        assert!(if let ErrorValue::InvalidToken(point, token, _) = result.unwrap_err() {
             point == "generate" && token == TokenType::NewLine.to_string()
        } else {
            false
        })
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
        assert!(match result {
            Err(ErrorValue::ErrorMessage(message, _)) => message.starts_with("Next pattern is nothing in patterns"),
            Err(_) => false,
            Ok(_) => false,
        })
    }
}