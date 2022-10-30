use proc_macro2::{
    TokenStream,
    Span
};
use syn::{
    parse::ParseStream,
    Result,
    Token,
    Ident, LitStr, LitInt, Error,
};
use quote::quote;

pub(crate) fn convert_zatlin(input: ParseStream) -> Result<TokenStream> {
    let mut tokens: Vec<TokenStream> = vec![];
    let token_list_name = Ident::new("__t_i",Span::call_site());

    while !input.is_empty() {
        let head = input.lookahead1();

        if head.peek(Token![%]) {
            input.parse::<Token![%]>()?;
            tokens.push(quote!{
                #token_list_name.push("%");
            });
            tokens.push(convert_expression(&token_list_name, input)?);
            input.parse::<Token![;]>()?;
            tokens.push(quote!{
                #token_list_name.push(";");
            });
        } else if head.peek(Ident) {
            let variable_name = input.parse::<Ident>()?.to_string();
            input.parse::<Token![=]>()?;
            tokens.push(quote! {
                #token_list_name.append(&mut vec![
                    #variable_name,
                    "="
                ]);
            });
            tokens.push(convert_expression(&token_list_name, input)?);
            input.parse::<Token![;]>()?;
            tokens.push(quote!{
                #token_list_name.push(";");
            });
        }
    }

    Ok(quote!{
        {
            let mut #token_list_name: std::vec::Vec<&str> = vec![];
            #(#tokens)*
            zatlin_internal::ZatlinData::try_from(#token_list_name)
        }
    })
}

fn convert_expression(key: &Ident, input: ParseStream) -> Result<TokenStream> {
    let mut patterns = Vec::new();
    let mut has_extract = false;
    patterns.push(convert_pattern(key, input, false)?);

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
            patterns.push(quote!{ #key.push("|"); });
            patterns.push(convert_pattern(key, input, false)?);
        } else {
            return Err(Error::new(input.span(), "invalid token"));
        }
    }

    if has_extract {
        patterns.push(quote!{ #key.push("-"); });
        if input.is_empty() {
            return Err(Error::new(input.span(), "end of statement in Exclude Pattern"));
        }

        if let Ok(_) = input.parse::<Token![^]>() {
            patterns.push(quote!{ #key.push("^"); });
        }

        let pattern = match convert_pattern(key, input, true) {
            Ok(t) => t,
            Err(_) => return Err(Error::new(input.span(), "invalid Exclude Pattern")),
        };
        patterns.push(pattern);

        if let Ok(_) = input.parse::<Token![^]>() {
            patterns.push(quote!{ #key.push("^"); });
        }

        while !input.is_empty() {
            let head = input.lookahead1();
    
            if head.peek(Token![;]) {
                break;
            } else if head.peek(Token![|]) {
                input.parse::<Token![|]>()?;

                if let Ok(_) = input.parse::<Token![^]>() {
                    patterns.push(quote!{ #key.push("^"); });
                }

                patterns.push(quote!{ #key.push("|"); });
                patterns.push(convert_pattern(key, input, true)?);

                if let Ok(_) = input.parse::<Token![^]>() {
                    patterns.push(quote!{ #key.push("^"); });
                }
            } else {
                return Err(Error::new(input.span(), "invalid token"));
            }

        }
    }

    Ok(quote!{
        #(#patterns)*
    })
}

fn convert_pattern(key: &Ident, input: ParseStream, is_exclude: bool) -> Result<TokenStream> {
    let mut values = Vec::new();

    while !input.is_empty() {
        let head = input.lookahead1();

        if head.peek(Ident) {
            if is_exclude {
                return Err(Error::new(input.span(), "cannot use variable in Exclude Pattern."));
            } else {
                let variable_name = input.parse::<Ident>()?.to_string();
                values.push(quote!{ #variable_name });
            }
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
            #key.append(&mut vec![
                #(#values),*
            ]);
        })
    }
}
