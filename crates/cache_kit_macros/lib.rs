use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Attribute, FnArg, ImplItem, ItemImpl, Pat, parse_macro_input};

#[proc_macro_attribute]
pub fn cached_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemImpl);
    let mut cached_methods = vec![];
    let self_ty = &input.self_ty;

    for item in &input.items {
        if let ImplItem::Fn(func) = item {
            let has_cached_attr = func.attrs.iter().any(|attr| is_cached_attr(attr));
            if has_cached_attr {
                let asyncness = &func.sig.asyncness;
                let name = &func.sig.ident;
                let output = match &func.sig.output {
                    syn::ReturnType::Default => quote! {},
                    syn::ReturnType::Type(_, ty) => quote! { -> #ty },
                };
                let inputs = &func.sig.inputs;
                let cached_name = format_ident!("cached_{}", name);
                let clean_cached_name = format_ident!("clean_cached_{}", name);
                let args: Vec<_> = func
                    .sig
                    .inputs
                    .iter()
                    .filter_map(|arg| match arg {
                        FnArg::Typed(pat_type) => {
                            if let Pat::Ident(ident) = &*pat_type.pat {
                                Some(quote! { #ident })
                            } else {
                                None
                            }
                        }
                        _ => None,
                    })
                    .collect();
                cached_methods.push(quote! {
                    pub #asyncness fn #cached_name(#inputs) #output {
                        self.cache_provider.get_with((#(#args),*), |q| async {
                            tracing::info!("Calling cached method: {}.{}", stringify!(#self_ty), stringify!(#name));
                            self.#name(q).await
                        }).await
                    }

                    pub async fn #clean_cached_name(&self) -> ApplicationResult<()> {
                        self.cache_provider.clear().await
                    }
                });
            }
        }
    }

    let gen_cached_impl = if !cached_methods.is_empty() {
        quote! {
            impl #self_ty {
                #(#cached_methods)*
            }
        }
    } else {
        quote! {}
    };
    let output = quote! {
        #input
        #gen_cached_impl
    };

    output.into()
}

fn is_cached_attr(attr: &Attribute) -> bool {
    attr.path().is_ident("cached")
}
