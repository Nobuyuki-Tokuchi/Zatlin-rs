use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use error::ErrorValue;
use parser::DefineStruct;
use rand::prelude::*;

mod lexer;
mod parser;
mod error;
use crate::lexer::{lexer};
use crate::parser::*;

pub struct Zatlin {
    operators: RefCell<Vec<Statement>>,
    text: String,
}

impl Zatlin {
    pub fn new(text: &str) -> Self {
        Zatlin {
            operators: RefCell::new(Vec::new()),
            text: String::from(text),
        }
    }

    pub fn generate(&self) -> Result<String, ErrorValue> {
        if self.operators.borrow().is_empty() {
            let tokens = lexer(&self.text);
            *self.operators.borrow_mut() = parse(&tokens)?;
        }

        execute(&self.operators)
    }

    pub fn generate_with(&self, count: i64) -> Result<Vec<String>, ErrorValue> {
        (0..count).map(|_| self.generate()).collect()
    }
}

#[derive(Debug, Clone)]
struct VariableData {
    pub expression: Rc<Expression>,
}

impl VariableData {
    pub fn new(expression: &Rc<Expression>) -> Self {
        Self {
            expression: Rc::clone(&expression),
        }
    }
}

fn execute(operators: &RefCell<Vec<Statement>>) -> Result<String, ErrorValue> {
    let operators = operators.borrow();
    let mut variables: HashMap<String, VariableData> = HashMap::new();
    let mut random = rand::thread_rng();

    for operator in operators.iter() {
        match operator {
            Statement::Define(DefineStruct { name: key, expr }) => {
                let data = VariableData::new(&expr);
                variables.insert(key.to_string(), data);
            },
            Statement::Generate(expr) => {
                return execute_expression(&VariableData::new(&expr), &variables, &mut random)
            },
        };
    }

    Ok(String::default())
}

fn execute_expression(data: &VariableData, variables: &HashMap<String, VariableData>, random: &mut ThreadRng) -> Result<String, ErrorValue> {
    let max: usize = data.expression.patterns.iter().map(|x| x.count).sum();

    let result = loop {
        let mut rng = random.clone();
        let value = rng.gen_range(0..max);
    
        let mut sum: usize = 0;
        let mut pattern: Option<&Pattern> = None;
        for item in data.expression.patterns.iter() {
            sum = sum + item.count;
            if value < sum {
                pattern = Some(item);
                break;
            }
        }
    
        let pattern = match pattern {
            Some(v) => v,
            None => return Err(ErrorValue::NotFoundPattern),
        };
        let mut rng = rng;
        let result = execute_pattern(&pattern, &variables, &mut rng)?;

        if !contains_excludes(&data.expression.excludes, &result) {
            break result;
        }
    };
    Ok(result)
}

fn contains_excludes(excludes: &Vec<Pattern>, result: &str) -> bool {
    excludes.iter().any(|x| {
        let check = x.values.iter().fold(String::default(), |acc, x| acc + match x { Value::Literal(s) => s, _ => "" });
        match x.mode {
            ExtractMode::None => result.contains(&check),
            ExtractMode::Forward => result.starts_with(&check),
            ExtractMode::Backward => result.ends_with(&check),
            ExtractMode::Exact => result == check,
        }
    })
}

fn execute_pattern(pattern: &Pattern, variables: &HashMap<String, VariableData>, random: &mut ThreadRng) -> Result<String, ErrorValue> {
    let mut result = String::default();
    let mut random = random;

    for item in pattern.values.iter() {
        let value = execute_value(&item, &variables, &mut random)?;
        result = result + &value;
    }

    Ok(result)
}

fn execute_value(value: &Value, variables: &HashMap<String, VariableData>, random: &mut ThreadRng) -> Result<String, ErrorValue> {
    match value {
        Value::Variable(key) => {
            if let Some(data) = variables.get(key) {
                let mut random = random;
                execute_expression(&data, &variables, &mut random)
            } else {
                Err(ErrorValue::NotFoundVariable(key.to_owned()))
            }
        },
        Value::Literal(val) => Ok(val.to_owned()),
    }
}

#[cfg(test)]
mod generate_test {
    use crate::error::ErrorValue;

    fn execute(s: &str) -> Result<Vec<String>, ErrorValue> {
        let zatlin = crate::Zatlin::new(s);
        zatlin.generate_with(32)
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
        
        match &result {
            Ok(value) => {
                println!("{}", value.join(" "));
            },
            Err(message) => {
                println!("{}", message);
            },
        }
        assert!(result.is_ok());
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
        
        match &result {
            Ok(value) => {
                println!("{}", value.join(" "));
            },
            Err(message) => {
                println!("{}", message);
            },
        }
        assert!(result.is_ok());
    }


    #[test]
    fn undefined_variable() {
        let result = execute(r#"
        C = "p" | "f" | "t" | "s" | "k" | "h";
        V = "a" | "i" | "u"

        # 'X' of variable is not defined.
        % C V | C V C | X | V C | V C V;
        "#);

        match &result {
            Ok(value) => {
                println!("{}", value.join(" "));
            },
            Err(message) => {
                println!("{}", message);
            },
        }
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ErrorValue::NotFoundVariable(String::from("X")))
    }
}