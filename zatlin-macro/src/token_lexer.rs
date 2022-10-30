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

    while !input.is_empty() {
        let head = input.lookahead1();

        if head.peek(Token![%]) {
            input.parse::<Token![%]>()?;
            tokens.push(quote!{
                "%"
            });
            tokens.push(convert_expression(input)?);
            input.parse::<Token![;]>()?;
            tokens.push(quote!{
                ";"
            });
        } else if head.peek(Ident) {
            let variable_name = input.parse::<Ident>()?.to_string();
            let binds = if let Ok(_) = input.parse::<Token![:]>() {
                Some(convert_binds(input)?)
            } else {
                None
            };
            input.parse::<Token![=]>()?;

            if let Some(binds) = binds {
                tokens.push(quote!{
                    #variable_name
                });
                tokens.push(binds);
                tokens.push(quote! {
                    "="
                });
            } else {
                tokens.push(quote! {
                    #variable_name,
                    "="
                });
            }
            tokens.push(convert_expression(input)?);
            input.parse::<Token![;]>()?;
            tokens.push(quote!{
                ";"
            });
        }
    }

    let token_list_name = Ident::new("__t_i",Span::call_site());
    Ok(quote!{
        {
            let #token_list_name: std::vec::Vec<&str> = vec![
                #(#tokens),*
            ];
            zatlin_internal::ZatlinData::try_from(#token_list_name)
        }
    })
}

fn convert_expression(input: ParseStream) -> Result<TokenStream> {
    let mut patterns = Vec::new();
    let mut has_extract = false;
    patterns.push(convert_pattern(input, false)?);

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
            patterns.push(convert_pattern(input, false)?);
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

        let pattern = match convert_pattern(input, true) {
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
                patterns.push(convert_pattern(input, true)?);

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

fn convert_pattern(input: ParseStream, is_exclude: bool) -> Result<TokenStream> {
    let values = convert_inner_pattern(input, is_exclude)?;

    if values.is_empty() {
        Err(Error::new(input.span(), "no pattern"))
    } else {
        Ok(quote! {
            #(#values),*
        })
    }
}

fn convert_inner_pattern(input: ParseStream, is_exclude: bool) -> Result<Vec<TokenStream>> {
    let mut values = Vec::new();

    while !input.is_empty() {
        let head = input.lookahead1();

        if head.peek(syn::token::Paren) {
            let content;
            syn::parenthesized!(content in input);
            let inner = convert_expression(&content)?;
            values.push(quote!{ "(" });
            values.push(inner);
            values.push(quote!{ ")" });
        }
        else if head.peek(Ident) {
            if is_exclude {
                return Err(Error::new(input.span(), "cannot use variable in Exclude Pattern."));
            } else {
                let variable_name = input.parse::<Ident>()?.to_string();
                values.push(quote!{ #variable_name });
            }
        } else if head.peek(LitStr) {
            let value = "\"".to_owned() + &input.parse::<LitStr>()?.value() + "\"";
            values.push(quote!{ #value });
        } else {
            if head.peek(LitInt) {
                let count = String::from(input.parse::<LitInt>()?.base10_digits());
                values.push(quote!{ #count });
            }
            break;
        }
    }

    Ok(values)
}

fn convert_binds(input: ParseStream) -> Result<TokenStream> {
    let mut bind_list = Vec::new();
    bind_list.push(convert_bind_statement(input)?);

    while !input.is_empty() && !input.peek(Token![=]) {
        input.parse::<Token![,]>()?;
        bind_list.push(quote!{ "," });
        bind_list.push(convert_bind_statement(input)?);
    }

    Ok(quote!{
        ":",
        #(#bind_list),*
    })
}

fn convert_bind_statement(input: ParseStream) -> Result<TokenStream> {
    let local_variable = input.parse::<Ident>()?.to_string();
    input.parse::<Token![<-]>()?;
    let destruct_variable = input.parse::<Ident>()?.to_string();

    Ok(quote!{
        #local_variable,
        "<-",
        #destruct_variable
    })
}