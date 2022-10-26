use std::collections::HashMap;
use std::rc::Rc;
use rand::prelude::*;

use zatlin_internal::{parser::*, ZatlinData};
pub use zatlin_internal::error::ErrorValue;

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
        let data = ZatlinData::new_str(text)?;
        execute(data.get_statements_ref(), self.retry_count)
    }

    pub fn generate_by(&self, data: &ZatlinData) -> Result<String, ErrorValue> {
        execute(data.get_statements_ref(), self.retry_count)
    }

    pub fn generate_many(&self, text: &str, count: u32) -> Vec<Result<String, ErrorValue>> {
        let data = match ZatlinData::new_str(text) {
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
        ZatlinData::new_str(text)
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
    let mut random = rand::thread_rng();

    for operator in operators.iter() {
        match operator {
            Statement::Define(DefineStruct { name: key, destruct_variables: local_variables, expr }) => {
                let data = VariableData::new(&local_variables, &expr, retry_count);
                variables.insert(key.to_string(), data);
            },
            Statement::Generate(expr) => {
                return execute_expression(&VariableData::without_destruct(&expr, retry_count), &variables, &mut random)
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
        let result = execute_pattern(&pattern, &variables, &mut rng, data.retry_count)?;

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

fn execute_pattern(pattern: &Pattern, variables: &HashMap<String, VariableData>, random: &mut ThreadRng, retry_count: u32) -> Result<String, ErrorValue> {
    let mut result = String::default();
    let mut random = random;

    for item in pattern.values.iter() {
        let value = execute_value(&item, &variables, &mut random, retry_count)?;
        result = result + &value;
    }

    Ok(result)
}

fn execute_value(value: &Value, variables: &HashMap<String, VariableData>, random: &mut ThreadRng, retry_count: u32) -> Result<String, ErrorValue> {
    match value {
        Value::Variable(key) => {
            if let Some(data) = variables.get(key) {
                let mut random = random;
                let append_variables = append_destruct_variables(data, variables, &mut random)?;
                execute_expression(&data, &append_variables, &mut random)
            } else {
                Err(ErrorValue::NotFoundVariable(key.to_owned()))
            }
        },
        Value::Literal(val) => Ok(val.to_owned()),
        Value::InnerPattern(patterns) => {
            let mut random = random;
            let expr = Rc::new(Expression { patterns: patterns.to_owned(), excludes: Vec::new() });
            let data = VariableData::without_destruct( &expr, retry_count);
            execute_expression(&data, &variables, &mut random)
        },
    }
}

fn append_destruct_variables(data: &VariableData, global: &HashMap<String, VariableData>, random: &mut ThreadRng) -> Result<HashMap<String, VariableData>, ErrorValue> {
    let mut variables: HashMap<String, VariableData> = HashMap::new();
    for (k, v) in global.iter() {
        variables.insert(k.to_owned(), v.clone());
    }

    for (k, v) in data.destruct_variables.iter() {
        let target = match global.get(v) {
            Some(local) => local,
            None => return Err(ErrorValue::NotFoundVariable(v.to_owned())),
        };
        let max: usize = target.expression.patterns.iter().map(|x| x.count).sum();

        let mut random = random.clone();
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

#[cfg(test)]
mod generate_test {
    use zatlin_internal::error::ErrorValue;

    fn execute(s: &str) -> Vec<Result<String, ErrorValue>> {
        let zatlin = crate::Zatlin::default();
        match crate::Zatlin::create_data(s) {
            Ok(data) => zatlin.generate_many_by(&data, 32),
            Err(error) => vec![Err(error)],
        }
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

    #[test]
    fn unofficial_circ() {
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
        
        Vxi = (Va | Ve | Vo) "i" | Vi ( "a" | "e" );
        Vxu = ( Va | Vo ) "u" | Vu ("e" | "i");
        Vx = Va | Ve | Vi | Vo | Vu | Vy | Vxi | Vxu;
        % Cs Vx Ce | Cs Vx Ce Cs Vx Ce - ^ ("" | "w" | "h" | "q" | "r" | "n" | "m") ("y" | "ý" | "ỳ" | "ÿ");
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
    fn unofficial_destruct_pattern() {
        let result = execute(r#"
        # test
        Ca = "p" | "b" | "f" | "v" | "m" | "t" | "d" | "s" | "z" | "n"
        Cb = "p" | "b" | "f" | "v" | "m" | "k" | "g" | "h"
        C = Ca | Cb
        Vi = "a" | "e" | "i"
        Vu = "a" | "o" | "u"
        V = Vi | Vu

        X : Vx <- V = C Vx C Vx;
        Y : Vx <- V, Cx <- C = Vx Cx Vx Cx | Cx Vx Cx Vx Cx;
        % V | V C | C V | C V C | X;
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
}