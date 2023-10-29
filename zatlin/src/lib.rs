use std::collections::HashMap;
use std::rc::Rc;
use rand::prelude::*;

mod lexer;
mod parser;
mod error;
mod data;
use crate::parser::*;
pub use crate::{error::Error, data::Data};

#[cfg(feature="use_macro")]
pub use zatlin_macro::zatlin;

pub struct Zatlin {
}

const DEFAULT_RETRY_COUNT: u32 = 100;

impl Default for Zatlin {
    fn default() -> Self {
        Self {
        }
    }
}

impl Zatlin {
    pub fn generate(&self, text: &str) -> Result<String, Error> {
        let data = Data::try_from(text)?;
        data.get_statements_ref().and_then(|x| execute(x))
    }

    pub fn generate_by(&self, data: &Data) -> Result<String, Error> {
        data.get_statements_ref().and_then(|x| execute(x))
    }

    pub fn generate_many(&self, text: &str, count: u32) -> Vec<Result<String, Error>> {
        let data = match Data::try_from(text) {
            Ok(data) => data,
            Err(error) => return vec![Err(error)]
        };
        self.generate_many_by(&data, count)
    }

    pub fn generate_many_by(&self, data: &Data, count: u32) -> Vec<Result<String, Error>> {
        let mut result = vec![];
        let mut i = 0;
        while i < count {
            result.push(data.get_statements_ref().and_then(|x| execute(x)));
            i += 1;
        }

        result
    }

    pub fn create_data(text: &str) -> Result<Data, Error> {
        Data::try_from(text)
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

fn execute(operators: &Vec<Statement>) -> Result<String, Error> {
    let mut variables: HashMap<String, VariableData> = HashMap::new();

    let mut retry_count = 1;
    loop {
        let mut result: Result<String, Error> = Ok(String::default());

        for operator in operators.iter() {
            match operator {
                Statement::Define(DefineStruct { name: key, expr }) => {
                    let data = VariableData::new(expr);
                    variables.insert(key.to_string(), data);
                },
                Statement::Generate(expr) => {
                    result = execute_expression(&VariableData::new(expr), &variables);
                    break;
                },
            };
        }

        if matches!(result, Ok(_)) { break result }

        retry_count += 1;
        if retry_count >= DEFAULT_RETRY_COUNT { break result }
    }
}

fn execute_expression(data: &VariableData, variables: &HashMap<String, VariableData>) -> Result<String, Error> {
    let max: f64 = data.expression.patterns.iter().map(|x| x.count).sum();
    let mut random = rand::thread_rng();

    let value = random.gen_range(0.0..max);

    let mut sum = 0.0;
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
        None => return Err(Error::NotFoundPattern),
    };
    let result = execute_pattern(&pattern, &variables)?;

    if !contains_excludes(&data.expression.excludes, &result) {
        Ok(result)
    } else {
        Err(Error::OverRetryCount)
    }
}

fn contains_excludes(excludes: &Exclude, result: &str) -> bool {
    if let Exclude::Regex(regex) = excludes {
        regex.is_match(result)
    } else {
        false
    }
}

fn execute_pattern(pattern: &Pattern, variables: &HashMap<String, VariableData>) -> Result<String, Error> {
    let mut result = String::default();

    for item in pattern.values.iter() {
        let value = execute_value(&item, &variables)?;
        result = result + &value;
    }

    Ok(result)
}

fn execute_value(value: &Value, variables: &HashMap<String, VariableData>) -> Result<String, Error> {
    match value {
        Value::Variable(key) => {
            if let Some(data) = variables.get(key) {
                execute_expression(&data, &variables)
            } else {
                Err(Error::NotFoundVariable(key.to_owned()))
            }
        },
        Value::Literal(val) => Ok(val.to_owned()),
        Value::InnerPattern(patterns) => {
            let expr = Rc::new(Expression { patterns: patterns.to_owned(), excludes: Exclude::Pattern(Vec::default()) });
            let data = VariableData::new(&expr);
            execute_expression(&data, &variables)
        },
    }
}
