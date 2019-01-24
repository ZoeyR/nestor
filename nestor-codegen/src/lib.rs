#![recursion_limit = "128"]

extern crate proc_macro;

use proc_macro::TokenStream;
use syn::export::TokenStream2;
use syn::spanned::Spanned;
use syn::token::Comma;
use syn::AttributeArgs;
use syn::Path;

use quote::{quote, quote_spanned};
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::{parse_macro_input, Expr, ItemFn, NestedMeta};

const COMMAND_PREFIX: &'static str = "nestor_command_handler_";
const ROUTE_ID_PREFIX: &'static str = "nestor_route_id_";

#[proc_macro_attribute]
pub fn command(macro_args: TokenStream, item: TokenStream) -> TokenStream {
    let macro_args = parse_macro_input!(macro_args as AttributeArgs);
    let route = match macro_args.get(0) {
        Some(NestedMeta::Literal(lit)) => quote! { Some(#lit) },
        _ => quote! { None },
    };

    let item = parse_macro_input!(item as ItemFn);
    let fn_name = &item.ident;
    let name = syn::Ident::new(
        &format!("{}{}", COMMAND_PREFIX, &item.ident),
        item.ident.span(),
    );

    let route_id = syn::Ident::new(
        &format!("{}{}", ROUTE_ID_PREFIX, &item.ident),
        item.ident.span(),
    );

    let args: Punctuated<_, Comma> = item
        .decl
        .inputs
        .iter()
        .map(|_| {
            let x = quote! {nestor::request::FromRequest::from_request(request)?}.into();
            syn::parse::<Expr>(x).unwrap()
        })
        .collect();

    let result = quote! {
        #[allow(non_upper_case_globals)]
        pub const #route_id: Option<&'static str> = #route;
        #[allow(non_camel_case_types)]
        pub struct #name;

        impl nestor::handler::CommandHandler for #name {
            fn handle<'a, 'r>(
                &'a self,
                request: &'a nestor::request::Request<'r>,
            ) -> Result<std::pin::Pin<Box<std::future::Future<Output = nestor::response::Outcome> + 'a>>, nestor::Error> {
                use std::pin::Pin;
                use nestor::response::IntoOutcome;
                use nestor::FutureExt;

                let fut = #fn_name(#args).map(|val| val.into_outcome());
                Ok(Box::pin(fut))
            }
        }

        #item
    };

    result.into()
}

fn prefix_last_segment(prefix: &'static str, path: &mut Path) {
    let mut last_seg = path.segments.last_mut().expect("syn::Path has segments");
    last_seg.value_mut().ident = syn::Ident::new(
        &format!("{}{}", prefix, &last_seg.value().ident),
        last_seg.value().ident.span(),
    )
}

fn _prefixed_vec(input: TokenStream) -> TokenStream2 {
    // Parse a comma-separated list of paths.
    let mut paths = <Punctuated<Path, Comma>>::parse_terminated
        .parse(input)
        .unwrap();
    let mut route_ids = paths.clone();

    // Prefix the last segment in each path with `prefix`.
    route_ids
        .iter_mut()
        .for_each(|p| prefix_last_segment(ROUTE_ID_PREFIX, p));
    paths
        .iter_mut()
        .for_each(|p| prefix_last_segment(COMMAND_PREFIX, p));

    // Return a `vec!` of the prefixed, mapped paths.
    let prefixed_mapped_paths = paths
        .iter()
        .zip(route_ids)
        .map(|(path, route_id)| quote_spanned!(path.span().into() => (#route_id,Box::new(#path))));

    quote!(vec![#(#prefixed_mapped_paths),*])
}

fn prefixed_vec(input: TokenStream) -> TokenStream {
    let vec = _prefixed_vec(input);

    quote!({
        let __vector: Vec<(Option<&'static str>, Box<dyn nestor::handler::CommandHandler>)> = #vec;
        __vector
    })
    .into()
}

#[proc_macro]
pub fn routes(input: TokenStream) -> TokenStream {
    prefixed_vec(input)
}
