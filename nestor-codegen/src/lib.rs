#![recursion_limit = "128"]

extern crate proc_macro;

use proc_macro::TokenStream;
use syn::spanned::Spanned;
use syn::AttributeArgs;

use quote::{quote, quote_spanned};
use syn::{parse_macro_input, ItemFn, NestedMeta};

const COMMAND_PREFIX: &'static str = "nestor_command_handler_";

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

    let args: Vec<_> = item
        .decl
        .inputs
        .iter()
        .map(|input| {
            let span = input.span();
            quote_spanned!(span=> nestor::request::FromRequest::from_request(request)?)
        })
        .collect();

    let (span, ty) = match &item.decl.output {
        syn::ReturnType::Default => (item.decl.output.span(), quote! {()}),
        syn::ReturnType::Type(_, ty) => (ty.span(), quote! {#ty}),
    };

    let function_call = if let Some(_) = item.asyncness {
        quote_spanned! {span=> {
            let fut = #fn_name(#(#args),*);
            async {
                let val = r#await!(fut);
                <#ty as IntoOutcome>::into_outcome(val)
            }
        }}
    } else {
        quote_spanned! {span => {
            let res = <#ty as IntoOutcome>::into_outcome(#fn_name(#(#args),*));
            async {res}
        }}
    };

    let result = quote! {
        #[allow(non_camel_case_types)]
        pub struct #name;

        nestor::inventory::submit!(#![crate = nestor] Box::new(#name) as Box<dyn nestor::handler::CommandHandler>);

        impl nestor::handler::CommandHandler for #name {
            fn route_id(&self) -> Option<&'static str> {
                #route
            }

            fn handle<'a, 'r>(
                &'a self,
                request: &'a nestor::request::Request<'r>,
            ) -> Result<std::pin::Pin<Box<std::future::Future<Output = nestor::response::Outcome> + 'a>>, nestor::Error> {
                use std::pin::Pin;
                use nestor::response::IntoOutcome;

                let fut = #function_call;
                Ok(Box::pin(fut))
            }
        }

        #item
    };

    result.into()
}
