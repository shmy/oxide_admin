use proc_macro::TokenStream;
mod event_subscriber;

#[proc_macro_derive(EventSubscriber)]
pub fn event_subscriber(input: TokenStream) -> TokenStream {
    event_subscriber::generate(input)
}
