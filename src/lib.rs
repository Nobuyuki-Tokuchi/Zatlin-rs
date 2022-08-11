use std::cell::RefCell;
use std::collections::HashMap;
use rand::prelude::*;

mod lexer;
mod parser;
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

    pub fn generate(&self) -> Result<String, String> {
        if self.operators.borrow().is_empty() {
            let tokens = lexer(&self.text);
            *self.operators.borrow_mut() = parse(&tokens)?;
        }

        execute(&self.operators)
    }

    pub fn generate_with(&self, count: i64) -> Result<Vec<String>, String> {
        (0..count).map(|_| self.generate()).collect()
    }
}

fn execute(operators: &RefCell<Vec<Statement>>) -> Result<String, String> {
    let operators = operators.borrow();
    let mut variables: HashMap<String, &Expression> = HashMap::new();
    let mut random = rand::thread_rng();

    for operator in operators.iter() {
        match operator {
            Statement::Define(key, expr) => variables.insert(key.to_string(), expr),
            Statement::Generate(expr) => return execute_expression(&expr, &variables, &mut random),
        };
    }

    Ok(String::default())
}

fn execute_expression(expr: &Expression, variables: &HashMap<String, &Expression>, random: &mut ThreadRng) -> Result<String, String> {
    let max: usize = expr.patterns.iter().map(|x| x.count).sum();

    let result = loop {
        let mut rng = random.clone();
        let value = rng.gen_range(0..max);
    
        let mut sum: usize = 0;
        let mut pattern: Option<&Pattern> = None;
        for item in expr.patterns.iter() {
            sum = sum + item.count;
            if value < sum {
                pattern = Some(item);
                break;
            }
        }
    
        let pattern = match pattern {
            Some(v) => v,
            None => return Err(String::from("Not found patterns")),
        };
        let mut rng = rng;
        let result = execute_pattern(&pattern, &variables, &mut rng)?;

        if !contains_excludes(&expr.excludes, &result) {
            break result;
        }
    };
    Ok(result)
}

fn contains_excludes(excludes: &Vec<ExcludePattern>, result: &str) -> bool {
    excludes.iter().any(|x| {
        let check = x.values.iter().fold(String::default(), |acc, x| acc + &x.value);
        match x.mode {
            ExtractMode::None => result.contains(&check),
            ExtractMode::Forward => result.starts_with(&check),
            ExtractMode::Backward => result.ends_with(&check),
            ExtractMode::Exact => result == check,
        }
    })
}

fn execute_pattern(pattern: &Pattern, variables: &HashMap<String, &Expression>, random: &mut ThreadRng) -> Result<String, String> {
    let mut result = String::default();
    let mut random = random;

    for item in pattern.values.iter() {
        let value = execute_value(&item, &variables, &mut random)?;
        result = result + &value;
    }

    Ok(result)
}

fn execute_value(value: &Value, variables: &HashMap<String, &Expression>, random: &mut ThreadRng) -> Result<String, String> {
    if value.is_variable {
        if let Some(expr) = variables.get(&value.value) {
            let mut random = random;
            execute_expression(expr, &variables, &mut random)
        } else {
            Err(format!("Not found variable: {}", value.value))
        }
    } else {
        Ok(value.value.clone())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn generate() {
        let zatlin = crate::Zatlin::new(r#"
        Cs = "b" | "p" | "f" | "v" | "d" | "t" | "s" | "z" | "c" | "j" | "g" | "k" | "h" | "q" | "r" | "w" | "n" | "m";
        Ce = "b" | "d" | "g" | "m" | "n" | "h";

        Va = "a" | "á" | "à" | "ä";
        Ve = "e" | "é" | "è" | "ë";
        Vi = "i" | "í" | "ì" | "ï";
        Vo = "o" | "ó" | "ò" | "ö";
        Vu = "u" | "ú" | "ù" | "ü";
        Vy = "y" | "ý" | "ỳ" | "ÿ";

        V1i = Va "i" | Ve "i" | Vo "i" | Vi "a" | Vi "e";
        V2i = Va "i" | Ve "i" | Vo "i" | Vi "a" | Vi "e";
        V3i = Va "i" | Ve "i" | Vo "i" | Vi "a" | Vi "e";
        V4i = Va "i" | Ve "i" | Vo "i" | Vi "a" | Vi "e";
        V1u = Va "u" | Vo "u" | Vu "e" | Vu "i";
        V2u = Va "u" | Vo "u" | Vu "e" | Vu "i";
        V3u = Va "u" | Vo "u" | Vu "e" | Vu "i";
        V4u = Va "u" | Vo "u" | Vu "e" | Vu "i";

        Vx1 = Va | Ve | Vi | Vo | Vu | Vy;
        Vx2 = V1i | V2i | V3i | V4i | V1u | V1u | V2u | V3u | V4u;
        VCx = Vx1 Ce | Vx2 Ce | Cs Vx1 Ce | Cs Vx2 Ce - "á" | "à" | "é" | "è" | "í" | "ì" | "ó" | "ò" | "ú" | "ù" | "ý" | "ỳ" ;

        % Vx1 | Vx2 | Cs Vx1 | Cs Vx2 | VCx - ^ "y" | ^ "ý" | ^ "ỳ" | ^ "ÿ" | ^ "wu" | ^ "wú" | ^ "wù" | ^ "wü" | ^ "hy" | ^ "hý" | ^ "hỳ" | ^ "hÿ" | ^ "qy" | ^ "qý" | ^ "qỳ" | ^ "qÿ" | ^ "ry" | ^ "rý" | ^ "rỳ" | ^ "rÿ" | ^ "ny" | ^ "ný" | ^ "nỳ" | ^ "nÿ" | ^ "my" | ^ "mý" | ^ "mỳ" | ^ "mÿ";
        "#);
        let result = zatlin.generate_with(32);
        assert!(match result {
            Ok(value) => {
                println!("{}", value.join(" "));
                true
            },
            Err(message) => {
                println!("{}", message);
                false
            },
        });
    }
}