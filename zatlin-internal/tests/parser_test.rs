use zatlin_internal::error::*;
use zatlin_internal::{lexer, lexer::TokenType, parser::*};

fn execute(s: &str) -> Result<Vec<Statement>, ErrorValue> {
    let tokens = lexer::lexer(s);
    parse(&tokens)
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