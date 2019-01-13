#![recursion_limit = "128"]

extern crate proc_macro;

use proc_macro::TokenStream;

use quote::quote;
use syn::{parse_macro_input, punctuated::Punctuated, token::Comma, Expr, ItemFn};

#[proc_macro_attribute]
pub fn command(_args: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as ItemFn);
    let name = &item.ident;

    let args: Punctuated<_, Comma> = item
        .decl
        .inputs
        .iter()
        .map(|_| {
            let x = quote! {irc_bot_types::FromRequest::from_request(request).unwrap()}.into();
            syn::parse::<Expr>(x).unwrap()
        })
        .collect();

    let result = quote! {
        pub struct irc_bot_command_handler;

        impl irc_bot_types::handler::CommandHandler for irc_bot_command_handler {
            fn handle<'a, 'r>(
                &'a self,
                request: &'a irc_bot_types::Request<'r>,
            ) -> Pin<Box<Future<Output = Result<Response, Error>> + 'a>> {
                use std::pin::Pin;
                let fut = #name(#args);
                Box::pin(fut)
            }
        }

        #item
    };

    result.into()
}
