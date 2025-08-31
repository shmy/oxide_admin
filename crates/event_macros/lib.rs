use proc_macro::TokenStream;
mod event_subscriber;

#[proc_macro_attribute]
pub fn event_subscriber(attr: TokenStream, input: TokenStream) -> TokenStream {
    event_subscriber::generate(attr, input)
}
