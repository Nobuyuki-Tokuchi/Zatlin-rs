use std::collections::HashMap;
use std::rc::Rc;
use error::ErrorValue;
use parser::DefineStruct;
use rand::prelude::*;

mod lexer;
mod parser;
mod error;
use crate::lexer::lexer;
use crate::parser::*;


pub struct Zatlin {
    retry_count: u32,
}

const DEFAULT_RETRY_COUNT: u32 = 100;

impl Default for Zatlin {
    fn default() -> Self {
        Self {
            retry_count: DEFAULT_RETRY_COUNT,
        }
    }
}

impl Zatlin {
    pub fn new(retry_count: u32) -> Self {
        Zatlin {
            retry_count: if retry_count == 0 {
                DEFAULT_RETRY_COUNT
            } else {
                retry_count
            },
        }
    }

    pub fn set_retry(&mut self, count: u32) {
        self.retry_count = if count < 1 {
            DEFAULT_RETRY_COUNT
        } else {
            count
        };
    }

    pub fn get_retry(&self) {
        self.retry_count;
    }

    pub fn generate(&self, text: &str) -> Result<String, ErrorValue> {
        let tokens = lexer(&text);
        let statements = parse(&tokens)?;
        execute(&statements, self.retry_count)
    }

    pub fn generate_many(&self, text: &str, count: u32) -> Vec<Result<String, ErrorValue>> {
        let tokens = lexer(&text);
        let statements = match parse(&tokens) {
            Ok(statements ) => statements,
            Err(error) => return vec![Err(error)],
        };

        let mut result = vec![];
        let mut i = 0;
        while i < count {
            result.push(execute(&statements, self.retry_count));
            i += 1;
        }

        result
    }
}

#[derive(Debug, Clone)]
struct VariableData {
    pub expression: Rc<Expression>,
    pub retry_count: u32,
}

impl VariableData {
    pub fn new(expression: &Rc<Expression>, retry_count: u32) -> Self {
        Self {
            expression: Rc::clone(&expression),
            retry_count: retry_count,
        }
    }
}

fn execute(operators: &Vec<Statement>, retry_count: u32) -> Result<String, ErrorValue> {
    let mut variables: HashMap<String, VariableData> = HashMap::new();
    let mut random = rand::thread_rng();

    for operator in operators.iter() {
        match operator {
            Statement::Define(DefineStruct { name: key, expr }) => {
                let data = VariableData::new(&expr, retry_count);
                variables.insert(key.to_string(), data);
            },
            Statement::Generate(expr) => {
                return execute_expression(&VariableData::new(&expr, retry_count), &variables, &mut random)
            },
        };
    }

    Ok(String::default())
}

fn execute_expression(data: &VariableData, variables: &HashMap<String, VariableData>, random: &mut ThreadRng) -> Result<String, ErrorValue> {
    let max: usize = data.expression.patterns.iter().map(|x| x.count).sum();

    let mut count: u32 = 0;
    loop {
        if count >= data.retry_count {
            break Err(ErrorValue::OverRetryCount);
        }

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
            break Ok(result);
        }

        count += 1;
    }
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

    fn execute(s: &str) -> Vec<Result<String, ErrorValue>> {
        let zatlin = crate::Zatlin::default();
        zatlin.generate_many(&s, 32)
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
        
        for item in result.iter() {
            match item {
                Ok(value) => {
                    print!("{} ", value);
                },
                Err(message) => {
                    print!("({}) ", message);
                },
            }
        }
        assert!(result.iter().all(|x| x.is_ok()));
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
        
        for item in result.iter() {
            match item {
                Ok(value) => {
                    print!("{} ", value);
                },
                Err(message) => {
                    print!("({}) ", message);
                },
            }
        }
        assert!(result.iter().all(|x| x.is_ok()));
    }

    #[test]
    fn undefined_variable() {
        let result = execute(r#"
        C = "p" | "f" | "t" | "s" | "k" | "h";
        V = "a" | "i" | "u"
        Y = C V

        # 'X' of variable is not defined.
        % X;
        "#);

        assert!(result.iter().all(|x| x.is_err()));
        assert!(result.iter().all(|x| if let Err(ErrorValue::NotFoundVariable(message)) = x { message == "X" } else { false }))
    }

    #[test]
    fn over_retry_count() {
        let result = execute(r#"
        C = "p" | "f" | "t" | "s" | "k" | "h";
        V = "a" | "i" | "u"

        # Retry count is over limit.
        % C V - "a" ^ | "i" ^ | "u" ^;
        "#);

        assert!(result.iter().all(|x| x.is_err()));
        assert!(result.iter().all(|x| if let Err(ErrorValue::OverRetryCount) = x { true } else { false }))
    }
}