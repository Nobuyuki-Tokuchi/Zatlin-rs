
use zatlin_internal::lexer::*;

fn execute(s: &str) -> Vec<TokenType> {
    lexer(&s)
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
    
    let unknown_tokens: Vec<(usize, &TokenType)> = result.iter().enumerate().filter(|(_, x)| {
        match x {
            TokenType::Unknown(_) => true,
            _ => false
        }
    }).collect();

    println!("{:?}", unknown_tokens);
    assert!(unknown_tokens.is_empty());
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
    
    let unknown_tokens: Vec<(usize, &TokenType)> = result.iter().enumerate().filter(|(_, x)| {
        match x {
            TokenType::Unknown(_) => true,
            _ => false
        }
    }).collect();

    println!("{:?}", unknown_tokens);
    assert!(unknown_tokens.is_empty());
}