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
    pub destruct_variables: Rc<HashMap<String, String>>,
    pub expression: Rc<Expression>,
}

impl VariableData {
    pub fn new(destruct_variables: &Rc<HashMap<String, String>>, expression: &Rc<Expression>) -> Self {
        Self {
            destruct_variables: Rc::clone(&destruct_variables),
            expression: Rc::clone(&expression),
        }
    }

    pub fn without_destruct(expression: &Rc<Expression>) -> Self {
        Self {
            destruct_variables: Rc::new(HashMap::new()),
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
            Statement::Define(DefineStruct { name: key, destruct_variables: local_variables, expr }) => {
                let data = VariableData::new(&local_variables, &expr);
                variables.insert(key.to_string(), data);
            },
            Statement::Generate(expr) => {
                return execute_expression(&VariableData::without_destruct(&expr), &variables, &mut random)
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
            let data = VariableData::without_destruct( &expr);
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
        variables.insert(k.to_owned(), VariableData::new(&(target.destruct_variables), &expression));
    }

    Ok(variables)
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
}