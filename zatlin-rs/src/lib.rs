use std::collections::HashMap;
use std::rc::Rc;
use rand::prelude::*;

use zatlin_internal::parser::*;
pub use zatlin_internal::{error::ErrorValue, ZatlinData};

#[cfg(feature="use_macro")]
pub use zatlin_macro::zatlin;

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

    pub fn get_retry(&self) -> u32 {
        self.retry_count
    }

    pub fn generate(&self, text: &str) -> Result<String, ErrorValue> {
        let data = ZatlinData::try_from(text)?;
        execute(data.get_statements_ref(), self.retry_count)
    }

    pub fn generate_by(&self, data: &ZatlinData) -> Result<String, ErrorValue> {
        execute(data.get_statements_ref(), self.retry_count)
    }

    pub fn generate_many(&self, text: &str, count: u32) -> Vec<Result<String, ErrorValue>> {
        let data = match ZatlinData::try_from(text) {
            Ok(data) => data,
            Err(error) => return vec![Err(error)]
        };
        self.generate_many_by(&data, count)
    }

    pub fn generate_many_by(&self, data: &ZatlinData, count: u32) -> Vec<Result<String, ErrorValue>> {
        let mut result = vec![];
        let mut i = 0;
        while i < count {
            result.push(execute(data.get_statements_ref(), self.retry_count));
            i += 1;
        }

        result
    }

    pub fn create_data(text: &str) -> Result<ZatlinData, ErrorValue> {
        ZatlinData::try_from(text)
    }
}

#[derive(Debug, Clone)]
struct VariableData {
    pub destruct_variables: Rc<HashMap<String, String>>,
    pub expression: Rc<Expression>,
    pub retry_count: u32,
}

impl VariableData {
    pub fn new(destruct_variables: &Rc<HashMap<String, String>>, expression: &Rc<Expression>, retry_count: u32) -> Self {
        Self {
            destruct_variables: Rc::clone(&destruct_variables),
            expression: Rc::clone(&expression),
            retry_count,
        }
    }

    pub fn without_destruct(expression: &Rc<Expression>, retry_count: u32) -> Self {
        Self {
            destruct_variables: Rc::new(HashMap::new()),
            expression: Rc::clone(&expression),
            retry_count,
        }
    }
}

fn execute(operators: &Vec<Statement>, retry_count: u32) -> Result<String, ErrorValue> {
    let mut variables: HashMap<String, VariableData> = HashMap::new();

    for operator in operators.iter() {
        match operator {
            Statement::Define(DefineStruct { name: key, destruct_variables: local_variables, expr }) => {
                let data = VariableData::new(&local_variables, &expr, retry_count);
                variables.insert(key.to_string(), data);
            },
            Statement::Generate(expr) => {
                return execute_expression(&VariableData::without_destruct(&expr, retry_count), &variables)
            },
        };
    }

    Ok(String::default())
}

fn execute_expression(data: &VariableData, variables: &HashMap<String, VariableData>) -> Result<String, ErrorValue> {
    let max: usize = data.expression.patterns.iter().map(|x| x.count).sum();
    let mut random = rand::thread_rng();

    let mut count: u32 = 0;
    loop {
        if count >= data.retry_count {
            break Err(ErrorValue::OverRetryCount);
        }

        let value = random.gen_range(0..max);
    
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
        let result = execute_pattern(&pattern, &variables, data.retry_count)?;

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

fn execute_pattern(pattern: &Pattern, variables: &HashMap<String, VariableData>, retry_count: u32) -> Result<String, ErrorValue> {
    let mut result = String::default();

    for item in pattern.values.iter() {
        let value = execute_value(&item, &variables, retry_count)?;
        result = result + &value;
    }

    Ok(result)
}

fn execute_value(value: &Value, variables: &HashMap<String, VariableData>, retry_count: u32) -> Result<String, ErrorValue> {
    match value {
        Value::Variable(key) => {
            if let Some(data) = variables.get(key) {
                let append_variables = append_destruct_variables(data, variables)?;
                execute_expression(&data, &append_variables)
            } else {
                Err(ErrorValue::NotFoundVariable(key.to_owned()))
            }
        },
        Value::Literal(val) => Ok(val.to_owned()),
        Value::InnerPattern(patterns) => {
            let expr = Rc::new(Expression { patterns: patterns.to_owned(), excludes: Vec::new() });
            let data = VariableData::without_destruct( &expr, retry_count);
            execute_expression(&data, &variables)
        },
    }
}

fn append_destruct_variables(data: &VariableData, global: &HashMap<String, VariableData>) -> Result<HashMap<String, VariableData>, ErrorValue> {
    let mut variables: HashMap<String, VariableData> = HashMap::new();
    let mut random = rand::thread_rng();

    for (k, v) in global.iter() {
        variables.insert(k.to_owned(), v.clone());
    }

    for (k, v) in data.destruct_variables.iter() {
        let target = match global.get(v) {
            Some(local) => local,
            None => return Err(ErrorValue::NotFoundVariable(v.to_owned())),
        };
        let max: usize = target.expression.patterns.iter().map(|x| x.count).sum();
        let value = random.gen_range(0..max);
    
        let mut sum: usize = 0;
        let mut pattern: Option<Pattern> = None;
        for item in target.expression.patterns.iter() {
            sum = sum + item.count;
            if value < sum {
                pattern = Some(item.clone());
                break;
            }
        }
    
        let pattern = match pattern {
            Some(v) => v,
            None => return Err(ErrorValue::NotFoundPattern),
        };

        let expression = Rc::new(Expression { patterns: vec![pattern], excludes: target.expression.excludes.clone() });
        variables.insert(k.to_owned(), VariableData::new(&(target.destruct_variables), &expression, data.retry_count));
    }

    Ok(variables)
}
