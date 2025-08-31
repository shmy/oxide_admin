use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

pub(crate) fn generate(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = input.ident;
    quote! {
        inventory::submit! {
            crate::shared::event_subscriber::EventRegistry::new(|event_bus, provider| {
                let subscriber = provider.provide::<#ident>();
                event_bus.subscribe(subscriber);
            })
        }
    }
    .into()
}
