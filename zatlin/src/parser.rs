
use std::collections::HashMap;
use std::rc::Rc;

use regex::Regex;

use crate::lexer::{TokenType, Token};
use crate::error::Error;

#[derive(Debug, Clone)]
pub(crate) enum Statement {
    Define(DefineStruct),
    Generate(Rc<Expression>)
}

#[derive(Debug, Clone)]
pub(crate) struct DefineStruct {
    pub name: String,
    pub expr: Rc<Expression>
}

#[derive(Debug, Clone)]
pub(crate) struct Expression {
    pub patterns: Vec<Pattern>,
    pub excludes: Exclude,
}

#[derive(Debug, Clone)]
pub(crate) enum Exclude {
    Pattern(Vec<Pattern>),
    Regex(Regex),
}

#[derive(Debug, Clone)]
pub(crate) struct Pattern {
    pub values: Vec<Value>,
    pub count: f64,
    pub mode: ExtractMode,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ExtractMode {
    None,
    Forward,
    Backward,
    Exact,
}

impl Pattern {
    fn new(values: Vec<Value>, count: f64, mode: ExtractMode) -> Self {
        Self { values, count, mode }
    }
}

impl Exclude {
    fn is_empty(&self) -> bool {
        match self {
            Exclude::Pattern(patterns) => patterns.is_empty(),
            Exclude::Regex(regex) => regex.as_str().is_empty(),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum Value {
    Literal(String),
    Variable(String),
    InnerPattern(Vec<Pattern>),
}

pub(crate) fn parse(tokens: &Vec<Token>) -> Result<Vec<Statement>, Error> {
    let mut statements = vec![];
    
    let mut index = 0;
    let length = tokens.len();
    while index < length {
        if let Some(value) = tokens.get(index) {
            match &value.tokentype {
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
                    return Err(Error::UnknownToken(value.clone(), index))
                },
                TokenType::NewLine => {
                    index = index + 1
                }
                _ => {
                    return Err(Error::InvalidToken(String::from("statement"), value.to_string(), index))
                }
            }
        } else {
            return Err(Error::ErrorMessage(format!("End of token in statement. length: {}, index: {}", length, index), None))
        }
    }
    
    convert_statement_exclude(statements)
}



fn convert_statement_exclude(statements: Vec<Statement>) -> Result<Vec<Statement>, Error> {
    let mut exclude_regex = HashMap::default();
    let mut updated_statements: Vec<Statement> = Vec::default();

    for statement in statements.iter() {
        let updated_statement = match statement {
            Statement::Define(def_statement) => {
                let mut used_variables: Vec<String> = Vec::default();
                convert_define_exclude(def_statement, &statements, &mut exclude_regex, &mut used_variables)
            },
            Statement::Generate(expression) => {
                let mut used_variables: Vec<String> = Vec::default();
                convert_generate_exclude(Rc::clone(expression), &statements, &mut exclude_regex, &mut used_variables)
            },
        }?;

        updated_statements.push(updated_statement);
    }

    Ok(updated_statements)
}

fn convert_generate_exclude(expression: Rc<Expression>, statements: &[Statement], exclude_regex: &mut HashMap<String, Regex>, used_variables: &mut Vec<String>) -> Result<Statement, Error> {
    if expression.excludes.is_empty() {
        Ok(Statement::Generate(expression))
    } else {
        let expr = expression.as_ref();
        let patterns = expr.patterns.clone();
        let excludes = convert_exclude(&expr.excludes, statements, exclude_regex, used_variables)?;
        
        Ok(Statement::Generate(Rc::new(Expression { patterns, excludes })))
    }
}

fn convert_define_exclude(def_statement: &DefineStruct, statements: &[Statement], exclude_regex: &mut HashMap<String, Regex>, used_variables: &mut Vec<String>) -> Result<Statement, Error> {
    if def_statement.expr.excludes.is_empty() {
        Ok(Statement::Define(def_statement.clone()))
    } else {
        let expr = def_statement.expr.as_ref();
        let patterns = expr.patterns.clone();
        let excludes = convert_exclude(&expr.excludes, statements, exclude_regex, used_variables)?;
    
        Ok(Statement::Define(DefineStruct { name: def_statement.name.clone(), expr: Rc::new(Expression { patterns, excludes }) }))
    }
}

fn convert_exclude(excludes: &Exclude, statements: &[Statement], exclude_regex: &mut HashMap<String, Regex>, used_variables: &mut Vec<String>) -> Result<Exclude, Error> {
    
    if let Exclude::Pattern(patterns) = excludes {
        let mut updated_excludes: Vec<String> = Vec::default();
        for pattern in patterns.iter() {
            updated_excludes.push(convert_pattern(pattern, statements, exclude_regex, used_variables, false)?);
        }

        match Regex::new(updated_excludes.join("|").as_str()) {
            Ok(regex) => Ok(Exclude::Regex(regex)),
            Err(_) => Err(Error::ErrorMessage("Invalid Exclude".to_owned(), None))
        }
    } else {
        Ok(excludes.clone())
    }

}

fn convert_pattern(pattern: &Pattern, statements: &[Statement], exclude_regex: &mut HashMap<String, Regex>, used_variables: &mut Vec<String>, anonymous_pattern: bool) -> Result<String, Error> {
    let mut pattern_str = String::default();
    let values = convert_from_values(&pattern.values, statements, exclude_regex, used_variables, anonymous_pattern)?;

    if pattern.mode == ExtractMode::Forward || pattern.mode == ExtractMode::Exact {
        pattern_str.push('^');
    }

    for value in values.iter(){
        let regex = if anonymous_pattern {
            if value.contains("|") {
                format!("({})", value)
            } else {
                value.to_string()
            }
        } else {
            value.to_string()
        };
        pattern_str.push_str(regex.as_str());
    }

    if pattern.mode == ExtractMode::Exact || pattern.mode == ExtractMode::Backward {
        pattern_str.push('$');
    }

    Ok(pattern_str)
}

fn convert_from_values(values: &[Value], statements: &[Statement], exclude_regex: &mut HashMap<String, Regex>, used_variables: &mut Vec<String>, anonymous_pattern: bool) -> Result<Vec<String>, Error> {
    let mut values_str = Vec::default();

    for value in values.iter() {
        let s = match value {
            Value::Literal(s) => s.to_owned(),
            Value::Variable(v) => {
                if used_variables.contains(v) {
                    return Err(Error::ErrorMessage("".to_owned(), None));
                }
                used_variables.push(v.clone());

                if let Some(Statement::Define(DefineStruct { name:_, expr })) = statements.iter().find(|x| if let Statement::Define(DefineStruct { name, expr: _ }) = x { name == v } else { false }) {
                    let result: Vec<Result<String, Error>> = expr.patterns.iter().map(|x| {
                        convert_pattern(x, statements, exclude_regex, used_variables, anonymous_pattern)
                    }).collect();
                    used_variables.pop();

                    if let Some(err) = result.iter().find_map(|x| x.as_ref().err()) {
                        return Err(err.to_owned());
                    } else {
                        let result = result.iter().map(|x| {
                            x.as_ref().unwrap().as_str()
                        }).collect::<Vec<&str>>().join("|");

                        if result.contains("|") {
                            format!("({})", result)
                        } else {
                            result.to_string()
                        }
                    }
                } else {
                    used_variables.pop();
                    return Err(Error::NotFoundVariable(v.to_owned()));
                }
            },
            Value::InnerPattern(patterns) => {
                let mut updated_excludes: Vec<String> = Vec::default();
                for pattern in patterns.iter() {
                    updated_excludes.push(convert_pattern(pattern, statements, exclude_regex, used_variables, false)?);
                }
                
                let result = String::from(updated_excludes.join("|"));

                if result.contains("|") {
                    format!("({})", result)
                } else {
                    result.to_string()
                }
            },
        };

        values_str.push(s);
    }

    Ok(values_str)
}


fn parse_define(value: &str, tokens: &[Token], index: usize) -> Result<(Statement, usize), Error> {
    let next_index = index;
    let next_index = if let Some(next) = tokens.get(next_index) {
        match &next.tokentype {
            &TokenType::Equal => next_index + 1,
            _ => return Err(Error::InvalidToken(String::from("define variable"), next.to_string(), index))
        }
    } else {
        return Err(Error::EndOfToken(String::from("define variable"), index))
    };

    let (expr, next_index) = parse_expression(&tokens, next_index)?;

    if let Some(token) = tokens.get(next_index) {
        if TokenType::Semicolon == token.tokentype || TokenType::NewLine == token.tokentype {
            Ok((Statement::Define(DefineStruct { name: String::from(value), expr: Rc::new(expr) }), next_index + 1))
        } else {
            Err(Error::InvalidToken(String::from("expression of define variable"), token.to_string(), index))
        }
    } else {
        Err(Error::EndOfToken(String::from("expression of define variable"), index))
    }
}

fn parse_generate(tokens: &[Token], index: usize) -> Result<(Statement, usize), Error> {
    let (expr, next_index) = parse_expression(&tokens, index)?;

    if let Some(token) = tokens.get(next_index) {
        if TokenType::Semicolon == token.tokentype {
            Ok((Statement::Generate(Rc::new(expr)), next_index + 1))
        } else {
            Err(Error::InvalidToken(String::from("generate"), token.tokentype.to_string(), next_index))
        }
    } else {
        Err(Error::EndOfToken(String::from("generate"), next_index))
    }
}

fn parse_expression(tokens: &[Token], index: usize) -> Result<(Expression, usize), Error> {
    let (patterns, next_index) = parse_patterns(&tokens, index)?;

    let (excludes, next_index) = if let Some(TokenType::Minus) = tokens.get(next_index).map(|x| &x.tokentype) {
        parse_patterns(&tokens, next_index + 1)?
    } else {
        (Vec::new(), next_index)
    };

    Ok((Expression { patterns, excludes: Exclude::Pattern(excludes) }, next_index))
}

fn parse_patterns(tokens: &[Token], index: usize) -> Result<(Vec<Pattern>, usize), Error> {
    let (pattern, mut next_index) = parse_pattern(&tokens, index)?;
    let mut patterns = vec![pattern];

    loop {
        if let Some(value) = tokens.get(next_index) {
            if TokenType::Or == value.tokentype {
                next_index = next_index + 1;
            } else {
                break;
            }
        } else {
            return Err(Error::EndOfToken(String::from("patterns"), next_index))
        };

        if let Ok((pattern, index)) = parse_pattern(&tokens, next_index) {
            patterns.push(pattern);
            next_index = index;
        } else {
            return Err(Error::ErrorMessage(String::from("Next pattern is nothing in patterns"), Some(next_index)))
        }
    }

    Ok((patterns, next_index))
}

fn parse_pattern(tokens: &[Token], index: usize) -> Result<(Pattern, usize), Error> {
    let (is_prefix, next_index) = if let Some(token) = tokens.get(index) {
        match token.tokentype {
            TokenType::Circumflex => (true, index + 1),
            _ => (false, index),
        }
    } else {
        return Err(Error::EndOfToken(String::from("pattern (prefix)"), index))
    };
    
    let (values, next_index) = parse_values(&tokens, next_index)?;
    let (count, next_index) = match tokens.get(next_index) {
        Some(value) => {
            if let TokenType::Count(value) = value.tokentype {
                (value, next_index + 1)
            } else {
                (1.0, next_index)
            }
        },
        None => {
            return Err(Error::EndOfToken(String::from("pattern"), index))
        }
    };

    let (is_postfix, next_index) = if let Some(token) = tokens.get(next_index) {
        match token.tokentype {
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

    Ok((Pattern::new(values, count, mode), next_index))
}

fn parse_values(tokens: &[Token], index: usize) -> Result<(Vec<Value>, usize), Error> {
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

fn parse_value(tokens: &[Token], index: usize) -> Result<(Value, usize), Error> {
    if let Some(token) = tokens.get(index) {
        match &token.tokentype {
            TokenType::Value(value) => Ok((Value::Literal(value.to_owned()), index + 1)),
            TokenType::Variable(value) => Ok((Value::Variable(value.to_owned()), index + 1)),
            TokenType::LeftCirc => parse_inner_patterns(&tokens, index + 1),
            _ => Err(Error::InvalidToken(String::from("value"), token.to_string(), index)),
        }
    } else {
            Err(Error::EndOfToken(String::from("value"), index))
    }
}

fn parse_inner_patterns(tokens: &[Token], index: usize) -> Result<(Value, usize), Error> {
    let (patterns, next_index) = parse_patterns(&tokens, index)?;

    if let Some(token) = tokens.get(next_index) {
        if TokenType::RightCirc == token.tokentype {
            Ok((Value::InnerPattern(patterns), next_index + 1))
        } else {
            Err(Error::InvalidToken(String::from("inner patterns"), token.to_string(), index))
        }
    } else {
        Err(Error::EndOfToken(String::from("inner patterns"), next_index))
    }
}

#[cfg(test)]
mod parse_test {
    use crate::lexer::TokenType;

    use super::{Statement, Error};

    fn execute(s: &str) -> Result<Vec<Statement>, Error> {
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

        # % Cs Vx Ce | Cs Vx Ce Cs Vx Ce - ^ "y" | ^ "ý" | ^ "ỳ" | ^ "ÿ" | ^ "wu" | ^ "wú" | ^ "wù" | ^ "wü" | ^ "hy" | ^ "hý" | ^ "hỳ" | ^ "hÿ" | ^ "qy" | ^ "qý" | ^ "qỳ" | ^ "qÿ" | ^ "ry" | ^ "rý" | ^ "rỳ" | ^ "rÿ" | ^ "ny" | ^ "ný" | ^ "nỳ" | ^ "nÿ" | ^ "my" | ^ "mý" | ^ "mỳ" | ^ "mÿ";
        % Cs Vx Ce | Cs Vx Ce Cs Vx Ce - ^ ("y" | "ý" | "ỳ" | "ÿ") | ^ "wu" | ^ "wú" | ^ "wù" | ^ "wü" | ^ "hy" | ^ "hý" | ^ "hỳ" | ^ "hÿ" | ^ "qy" | ^ "qý" | ^ "qỳ" | ^ "qÿ" | ^ "ry" | ^ "rý" | ^ "rỳ" | ^ "rÿ" | ^ "ny" | ^ "ný" | ^ "nỳ" | ^ "nÿ" | ^ "my" | ^ "mý" | ^ "mỳ" | ^ "mÿ";
        "#);

        println!("{:?}", result);
        assert!(result.is_ok())
    }

    #[test]
    fn complex1() {
        let result = execute(r#"
        identifier = ("a" | "b" | "c") ("d" | "e") | "f" ("g" | "h")
        % identified identified - "b" | "c" | "d" | "f" | "g" | "h" ^;
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
    fn use_variable_in_exclude() {
        let result = execute(r#"
        # metapi
        Cs = "" | "b" | "p" | "f" | "v" | "d" | "t" | "s" | "z" | "c" | "j" | "g" | "k" | "h" | "q" | "r" | "w" | "n" | "m"
        Ce = "" | "b" | "d" | "g" | "m" | "n" | "h"

        Va = "a" | "á" | "à" | "ä";
        Ve = "e" | "é" | "è" | "ë";
        Vi = "i" | "í" | "ì" | "ï";
        Vo = "o" | "ó" | "ò" | "ö";
        Vu = "u" | "ú" | "ù" | "ü";
        Vy = "y" | "ý" | "ỳ" | "ÿ";

        Vxi = Va "i" | Ve "i" | Vo "i" | Vi "a" | Vi "e"
        Vxu = Va "u" | Vo "u" | Vu "e" | Vu "i"
        Vx = Va | Ve | Vi | Vo | Vu | Vy | Vxi | Vxu

        % Cs Vx Ce | Cs Vx Ce Cs Vx Ce - ^ Vy | ^ "w" Vu | ^ "h" Vy | ^ "q" Vy | ^ "r" Vy | ^ "n" Vy | ^ "m" Vy;
        "#);

        println!("{:?}", result);
        assert!(result.is_ok())
    }

    #[test]
    fn use_variable_in_exclude2() {
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

        Excludes = ("" | "h" | "q" | "r" | "n" | "m") Vy | "w" Vu
        % Cs Vx Ce | Cs Vx Ce Cs Vx Ce - ^ Excludes;
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
        assert!(if let Error::InvalidToken(point, token, _) = result.unwrap_err() {
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
            Err(Error::ErrorMessage(message, _)) => message.starts_with("Next pattern is nothing in patterns"),
            Err(_) => false,
            Ok(_) => false,
        })
    }
}