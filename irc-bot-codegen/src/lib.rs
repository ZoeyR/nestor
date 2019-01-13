#![recursion_limit = "128"]

extern crate proc_macro;

use proc_macro::TokenStream;
use syn::export::TokenStream2;
use syn::spanned::Spanned;
use syn::token::Comma;
use syn::Path;

use quote::{quote, quote_spanned};
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::{parse_macro_input, Expr, Ident, ItemFn, Token};

const COMMAND_PREFIX: &'static str = "irc_bot_command_handler_";

#[proc_macro_attribute]
pub fn command(_args: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as ItemFn);
    let fn_name = &item.ident;
    let name = syn::Ident::new(
        &format!("{}{}", COMMAND_PREFIX, &item.ident),
        item.ident.span(),
    );

    let args: Punctuated<_, Comma> = item
        .decl
        .inputs
        .iter()
        .map(|_| {
            let x = quote! {irc_bot::request::FromRequest::from_request(request).unwrap()}.into();
            syn::parse::<Expr>(x).unwrap()
        })
        .collect();

    let result = quote! {
        pub struct #name;

        impl irc_bot::handler::CommandHandler for #name {
            fn handle<'a, 'r>(
                &'a self,
                request: &'a irc_bot::request::Request<'r>,
            ) -> Pin<Box<Future<Output = Result<Response, Error>> + 'a>> {
                use std::pin::Pin;
                let fut = #fn_name(#args);
                Box::pin(fut)
            }
        }

        #item
    };

    result.into()
}

fn prefix_last_segment(path: &mut Path) {
    let mut last_seg = path.segments.last_mut().expect("syn::Path has segments");
    last_seg.value_mut().ident = syn::Ident::new(
        &format!("{}{}", COMMAND_PREFIX, &last_seg.value().ident),
        last_seg.value().ident.span(),
    )
}

fn _prefixed_vec(input: TokenStream) -> TokenStream2 {
    // Parse a comma-separated list of paths.
    let mut paths = <Punctuated<Path, Comma>>::parse_terminated
        .parse(input)
        .unwrap();

    // Prefix the last segment in each path with `prefix`.
    let names = paths.clone().into_iter().map(|name| {
        let last_seg = name.segments.last().expect("syn::Path has segments");
        last_seg.value().ident.to_string()
    });
    paths.iter_mut().for_each(|p| prefix_last_segment(p));

    // Return a `vec!` of the prefixed, mapped paths.
    let prefixed_mapped_paths = paths
        .iter()
        .zip(names)
        .map(|(path, name)| quote_spanned!(path.span().into() => (#name,Box::new(#path))));

    quote!(vec![#(#prefixed_mapped_paths),*])
}

fn prefixed_vec(input: TokenStream) -> TokenStream {
    let vec = _prefixed_vec(input);

    quote!({
        let __vector: Vec<(&'static str, Box<dyn irc_bot::handler::CommandHandler>)> = #vec;
        __vector
    })
    .into()
}

#[proc_macro]
pub fn routes(input: TokenStream) -> TokenStream {
    prefixed_vec(input)
}
