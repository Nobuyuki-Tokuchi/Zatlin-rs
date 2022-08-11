

#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    Unknown(String),
    Minus,
    Or,
    Equal,
    Circumflex,
    Percent,
    Semicolon,
    Value(String),
    Count(usize),
    Variable(String),
}

#[derive(PartialEq, Copy, Clone)]
enum TokenizeMode {
    Normal,
    String,
    Comment,
}

pub fn lexer(text: &str) -> Vec<TokenType> {
    let text = text.chars();

    let mut tokens: Vec<TokenType> = vec![];
    let mut buffer: Vec<char> = vec![];
    let mut mode = TokenizeMode::Normal;

    for c in text {
        if mode == TokenizeMode::Comment {
            if c == '\r' || c == '\n' {
                mode = TokenizeMode::Normal;
            }
        } else if c.is_ascii_whitespace() {
            if mode == TokenizeMode::String {
                buffer.push(c);
            } else {
                if !buffer.is_empty() {
                    let token = String::from_iter(buffer.iter());
                    tokens.push(get_value(&token));
                    buffer.clear();
                }
            }
        } else {
            match c {
                '\r' | '\n' => {
                    if !buffer.is_empty() {
                        let token = String::from_iter(buffer.iter());
                        tokens.push(get_value(&token));
                        buffer.clear();
                    }
                }
                '-' | '|' | ';' | '%' | '^' | '=' => {
                    if !buffer.is_empty() {
                        let token = String::from_iter(buffer.iter());
                        tokens.push(get_value(&token));
                        buffer.clear();
                    }

                    tokens.push(get_tokentype(c));
                },
                '#' => {
                    mode = TokenizeMode::Comment;
                },
                '"' => {
                    if mode == TokenizeMode::String {
                        mode = TokenizeMode::Normal;
                        buffer.push(c);

                        let token = String::from_iter(buffer.iter());
                        tokens.push(get_value(&token));
                        buffer.clear();
                    } else {
                        mode = TokenizeMode::String;
                        buffer.push(c);
                    }
                }
                _ => {
                    buffer.push(c);
                }
            }
        }
    }

    tokens
}

fn get_value(value: &str) -> TokenType {
    if let Ok(num) = value.parse() {
        TokenType::Count(num)
    } else if value.starts_with('"') && value.ends_with('"') {
        let len = value.len();
        if len > 2 {
            TokenType::Value(value[1..(value.len() - 1)].to_string())
        } else if len == 2 {
            TokenType::Value("".to_string())
        } else {
            TokenType::Unknown(value.to_string())
        }
    } else {
        TokenType::Variable(String::from(value))
    }
}

fn get_tokentype(value: char) -> TokenType {
    match value {
        '-' => TokenType::Minus,
        '|' => TokenType::Or,
        ';' => TokenType::Semicolon,
        '%' => TokenType::Percent,
        '^' => TokenType::Circumflex,
        '=' => TokenType::Equal,
        _ => TokenType::Unknown(String::from(value)),
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn lexer_test() {
        let text = String::from(r#"
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

        let tokens = crate::lexer::lexer(&text);
        println!("{:?}", tokens)
    }
}