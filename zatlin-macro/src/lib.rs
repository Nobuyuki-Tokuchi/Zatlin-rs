use proc_macro2::TokenStream;
use syn::{
    parse::Parser,
    Error,
};

mod token_lexer;
use crate::token_lexer::convert_zatlin;

#[proc_macro]
pub fn zatlin(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    zatlin_impl(tokens.into()).into()
}

fn zatlin_impl(tokens: TokenStream) -> TokenStream {
    convert_zatlin.parse2(tokens).unwrap_or_else(Error::into_compile_error)
}

#[cfg(test)]
mod zatlin_macro_test {
    use quote::quote;
    use crate::*;

    #[test]
    fn check() {
        let data = zatlin_impl(quote! {
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
        });

        println!("{}", data);
    }

    #[test]
    fn unofficial_circ_check() {
        let data = zatlin_impl(quote!{
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
        });

        println!("{}", data);
    }

    #[test]
    fn unofficial_destruct_pattern_check() {
        let data = zatlin_impl(quote!{
            Ca = "p" | "b" | "f" | "v" | "m" | "t" | "d" | "s" | "z" | "n";
            Cb = "p" | "b" | "f" | "v" | "m" | "k" | "g" | "h";
            C = Ca | Cb;
            Vi = "a" | "e" | "i";
            Vu = "a" | "o" | "u";
            V = Vi | Vu;
    
            X : Vx <- V = C Vx C Vx;
            Y : Vx <- V, Cx <- C = Vx Cx Vx Cx | Cx Vx Cx Vx Cx;
            % V | V C | C V | C V C | X;
        });

        println!("{}", data);
    }
}