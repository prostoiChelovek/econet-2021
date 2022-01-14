extern crate proc_macro;

use std::collections::HashSet as Set;

use proc_macro::TokenStream;
use syn::{
    punctuated::Punctuated,
    parse::{Parse, ParseStream, Result as ParseResult},
    Expr, Ident, Token,
    parse_macro_input, ExprTuple, ExprGroup
};

#[derive(Debug, Hash)]
struct PinArg {
    a: i8
    /*
    port: Expr,
    pin: Expr
        */
}

#[derive(Debug)]
struct Args {
    //pins: Set<PinArg>,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let vars = Punctuated::<Expr, Token![,]>::parse_terminated(input)?
            .into_iter()
            .filter_map(|x| match x {
                Expr::Tuple(t) => Some(t),
                _ => None
            })
            .map(|x| {
                x.elems.into_iter().filter_map(|e| match e {
                    Expr::Group(g) => Some(g),
                    _ => None
                })
                .map(|g| g.expr)
                .collect::<Vec<_>>()
            }).collect::<Vec<_>>();
        println!("{:?}", vars);

        Ok(Args {
            //pins: vars.into_iter().collect(),
        })

    }
}

#[proc_macro]
pub fn generate_pins_create(tokens: TokenStream) -> TokenStream {
    let args = parse_macro_input!(tokens as Args);

    println!("{:?}", args);

    quote::quote!(
        macro_rules! create_pins {
            ($gpio:ident) => {
            }
        }
    ).into()
}
