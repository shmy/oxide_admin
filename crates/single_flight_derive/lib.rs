use proc_macro::TokenStream;
mod single_flight;

#[proc_macro_attribute]
pub fn single_flight(attr: TokenStream, item: TokenStream) -> TokenStream {
    single_flight::generate(attr, item)
}
