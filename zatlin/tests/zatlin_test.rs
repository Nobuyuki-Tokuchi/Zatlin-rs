
use zatlin::{Zatlin, Error};

fn execute(s: &str) -> Vec<Result<String, Error>> {
    let zatlin = Zatlin::default();
    match Zatlin::create_data(s) {
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

    Vxi = Va "i" | Ve "i" | Vo "i" | Vi ("a" | "e")
    Vxu = Va "u" | Vo "u" | Vu ("e" | "i")
    Vx = Va | Ve | Vi | Vo | Vu | Vy | Vxi | Vxu

    % Cs Vx Ce | Cs Vx Ce Cs Vx Ce - ^ ("y" | "ý" | "ỳ" | "ÿ") | ^ "w" ("u" | "ú" | "ù" | "ü") | ^ ("h" | "q" | "r" | "n" | "m") ("y" | "ý" | "ỳ" | "ÿ");
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
    println!("");
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
    println!("");
    assert!(result.iter().all(|x| x.is_ok()));
}

#[test]
fn use_variable_in_exclude() {
    let result = execute(r#"
    # metapi
    Cs = "" | "b" | "p" | "f" | "v" | "d" | "t" | "s" | "z" | "c" | "j" | "g" | "k" | "h" | "q" | "r" | "w" | "n" | "m"
    Ce = "" | "b" | "d" | "g" | "m" | "n" | "h"

    Va = "a" | "á" | "à" | "ä";
    Ve = "e" | "é" | "è" | "ë";
    Vi = "i" | "í" | "ì" | "ï";
    Vo = "o" | "ó" | "ò" | "ö";
    Vu = "u" | "ú" | "ù" | "ü";
    Vy = "y" | "ý" | "ỳ" | "ÿ";

    Vxi = (Va | Ve | Vo) "i" | Vi ("a" | "e")
    Vxu = (Va | Vo) "u" | Vu ("e" | "i")
    Vx = Va | Ve | Vi | Vo | Vu | Vy | Vxi | Vxu

    % Cs Vx Ce | Cs Vx Ce Cs Vx Ce - ^ Vy | ^ "w" Vu | ^ "h" Vy | ^ "q" Vy | ^ "r" Vy | ^ "n" Vy | ^ "m" Vy;
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
    println!("");
    assert!(result.iter().all(|x| x.is_ok()));
}

#[test]
fn use_variable_in_exclude2() {
    let result = execute(r#"
    # metapi
    Cs = "" | "b" | "p" | "f" | "v" | "d" | "t" | "s" | "z" | "c" | "j" | "g" | "k" | "h" | "q" | "r" | "w" | "n" | "m"
    Ce = "" | "b" | "d" | "g" | "m" | "n" | "h"

    Va = "a" | "á" | "à" | "ä";
    Ve = "e" | "é" | "è" | "ë";
    Vi = "i" | "í" | "ì" | "ï";
    Vo = "o" | "ó" | "ò" | "ö";
    Vu = "u" | "ú" | "ù" | "ü";
    Vy = "y" | "ý" | "ỳ" | "ÿ";

    Vxi = Va "i" | Ve "i" | Vo "i" | Vi "a" | Vi "e"
    Vxu = Va "u" | Vo "u" | Vu "e" | Vu "i"
    Vx = Va | Ve | Vi | Vo | Vu | Vy | Vxi | Vxu

    Excludes = ("" | "h" | "q" | "r" | "n" | "m") Vy | "w" Vu
    % Cs Vx Ce | Cs Vx Ce Cs Vx Ce - ^ Excludes;
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
    println!("");
    assert!(result.iter().all(|x| x.is_ok()));
}

#[test]
fn use_variable_in_exclude3() {
    let result = execute(r#"
    # metapi
    Cs = "" | "b" | "p" | "f" | "v" | "d" | "t" | "s" | "z" | "c" | "j" | "g" | "k" | "h" | "q" | "r" | "w" | "n" | "m"
    Ce = "" | "b" | "d" | "g" | "m" | "n" | "h"

    Va = "a" | "á" | "à" | "ä";
    Ve = "e" | "é" | "è" | "ë";
    Vi = "i" | "í" | "ì" | "ï";
    Vo = "o" | "ó" | "ò" | "ö";
    Vu = "u" | "ú" | "ù" | "ü";
    Vy = "y" | "ý" | "ỳ" | "ÿ";

    Vxi = Va "i" | Ve "i" | Vo "i" | Vi "a" | Vi "e"
    Vxu = Va "u" | Vo "u" | Vu "e" | Vu "i"
    Vx = Va | Ve | Vi | Vo | Vu | Vy | Vxi | Vxu

    % Cs Vx Ce | Cs Vx Ce Cs Vx Ce - ^ (("" | "h" | "q" | "r" | "n" | "m") Vy | "w" Vu);
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
    println!("");
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
    assert!(result.iter().all(|x| if let Err(Error::NotFoundVariable(message)) = x { message == "X" } else { false }))
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
    assert!(result.iter().all(|x| if let Err(Error::OverRetryCount) = x { true } else { false }))
}
