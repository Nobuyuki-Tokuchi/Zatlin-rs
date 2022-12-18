use zatlin_internal::{lexer, lexer::TokenType};
use zatlin_internal::parser;
use zatlin_internal::parser::{Statement, DefineStruct};
use zatlin_internal::error::ErrorValue;

fn execute(s: &str) -> Result<Vec<Statement>, ErrorValue> {
    let tokens = lexer::lexer(s);
    parser::parse(&tokens)
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
    assert!(result.is_ok());
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
            Statement::Define(DefineStruct { name: _, destruct_variables: _, expr }) => {
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

    println!("{:?}", result);
    assert!(result.is_ok())
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

    println!("{:?}", result);
    assert!(result.is_ok())
}