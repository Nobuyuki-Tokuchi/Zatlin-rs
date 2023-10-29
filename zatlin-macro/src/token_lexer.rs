use proc_macro2::{
    TokenStream,
    Span
};
use syn::{
    parse::ParseStream,
    Result,
    Token, parenthesized,
    Ident, LitStr, LitInt, Error, token::Paren,
};
use quote::quote;

pub(crate) fn convert_zatlin(input: ParseStream) -> Result<TokenStream> {
    let mut tokens: Vec<TokenStream> = vec![];

    while !input.is_empty() {
        let head = input.lookahead1();

        if head.peek(Token![%]) {
            input.parse::<Token![%]>()?;
            tokens.push(quote!{ "%" });
            tokens.push(convert_expression(input)?);
            input.parse::<Token![;]>()?;
            tokens.push(quote!{ ";" });
        } else if head.peek(Ident) {
            let variable_name = input.parse::<Ident>()?.to_string();
            input.parse::<Token![=]>()?;
            tokens.push(quote! {
                #variable_name,
                "="
            });
            tokens.push(convert_expression(input)?);
            input.parse::<Token![;]>()?;
            tokens.push(quote!{ ";" });
        }
    }

    let token_list_name = Ident::new("__t_i",Span::call_site());
    Ok(quote!{
        {
            let mut #token_list_name: std::vec::Vec<&str> = vec![
                #(#tokens),*
            ];
            zatlin::Data::try_from(#token_list_name)
        }
    })
}

fn convert_expression(input: ParseStream) -> Result<TokenStream> {
    let mut patterns = Vec::new();
    let mut has_extract = false;
    patterns.push(convert_pattern(input)?);

    while !input.is_empty() {
        let head = input.lookahead1();

        if head.peek(Token![;]) {
            break;
        } else if head.peek(Token![-]) {
            input.parse::<Token![-]>()?;
            has_extract = true;
            break;
        } else if head.peek(Token![|]) {
            input.parse::<Token![|]>()?;
            patterns.push(quote!{ "|" });
            patterns.push(convert_pattern(input)?);
        } else {
            return Err(Error::new(input.span(), "invalid token"));
        }
    }

    if has_extract {
        patterns.push(quote!{ "-" });
        if input.is_empty() {
            return Err(Error::new(input.span(), "end of statement in Exclude Pattern"));
        }

        if let Ok(_) = input.parse::<Token![^]>() {
            patterns.push(quote!{ "^" });
        }

        let pattern = match convert_pattern(input) {
            Ok(t) => t,
            Err(_) => return Err(Error::new(input.span(), "invalid Exclude Pattern")),
        };
        patterns.push(pattern);

        if let Ok(_) = input.parse::<Token![^]>() {
            patterns.push(quote!{ "^" });
        }

        while !input.is_empty() {
            let head = input.lookahead1();
    
            if head.peek(Token![;]) {
                break;
            } else if head.peek(Token![|]) {
                input.parse::<Token![|]>()?;

                if let Ok(_) = input.parse::<Token![^]>() {
                    patterns.push(quote!{ "^" });
                }

                patterns.push(quote!{ "|" });
                patterns.push(convert_pattern(input)?);

                if let Ok(_) = input.parse::<Token![^]>() {
                    patterns.push(quote!{ "^" });
                }
            } else {
                return Err(Error::new(input.span(), "invalid token"));
            }

        }
    }

    Ok(quote!{
        #(#patterns),*
    })
}

fn convert_pattern(input: ParseStream) -> Result<TokenStream> {
    let mut values = Vec::new();

    while !input.is_empty() {
        let head = input.lookahead1();

        if head.peek(Paren) {
            let content;
            let _ = parenthesized!(content in input);
            let inner = convert_expression(&content)?;
            values.push(quote!{ "(", #inner, ")" });
        }
        else if head.peek(Ident) {
            let variable_name = input.parse::<Ident>()?.to_string();
            values.push(quote!{ #variable_name });
        } else if head.peek(LitStr) {
            let value = "\"".to_owned() + &input.parse::<LitStr>()?.value() + "\"";
            values.push(quote!{ #value })
        } else {
            if head.peek(LitInt) {
                let count = String::from(input.parse::<LitInt>()?.base10_digits());
                values.push(quote!{ #count });
            }
            break;
        }
    }

    if values.is_empty() {
        Err(Error::new(input.span(), "no pattern"))
    } else {
        Ok(quote!{
            #(#values),*
        })
    }
}
