
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
    
    let unknown_tokens: Vec<(usize, &TokenType)> = result.iter().enumerate().filter(|(_, x)| {
        match x {
            TokenType::Unknown(_) => true,
            _ => false
        }
    }).collect();
    let circ_check = result.iter().fold(0, |acc, x| {
        match x {
            TokenType::LeftCirc => acc + 1,
            TokenType::RightCirc => acc - 1,
            _ => acc
        }
    });

    println!("{:?}", unknown_tokens);
    assert!(unknown_tokens.is_empty());
    assert_eq!(circ_check, 0);
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
    
    let unknown_tokens: Vec<(usize, &TokenType)> = result.iter().enumerate().filter(|(_, x)| {
        match x {
            TokenType::Unknown(_) => true,
            _ => false
        }
    }).collect();

    println!("{:?}", unknown_tokens);
    assert!(unknown_tokens.is_empty());
}