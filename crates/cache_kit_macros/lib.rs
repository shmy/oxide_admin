use std::time::Duration;

use humantime::parse_duration;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Attribute, FnArg, ImplItem, ItemImpl, Lit, Pat, parse_macro_input};

struct CachedArgs {
    prefix: String,
    ttl: Duration,
}

#[proc_macro_attribute]
pub fn cached_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemImpl);
    let mut cached_methods = vec![];
    let self_ty = &input.self_ty;

    for item in &input.items {
        if let ImplItem::Fn(func) = item {
            let cached_attr = func
                .attrs
                .iter()
                .find(|attr| attr.path().is_ident("cached"));

            if let Some(cached_attr) = cached_attr {
                let cached_args = parse_cached_args(cached_attr);

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
                let prefix = cached_args.prefix;
                let ttl = cached_args.ttl.as_secs();
                cached_methods.push(quote! {
                    pub #asyncness fn #cached_name(#inputs) #output {
                        let cache_key = format!("{}:{}{}", cache_kit::CACHE_PREFIX, #prefix, cache_kit::encode_cache_key(&(#(#args),*)));
                        if let Some(cache) = self.cache.get(&cache_key).await {
                            return Ok(cache);
                        }
                        let value = self.#name((#(#args),*)).await?;
                        if let Err(err) = self
                            .cache
                            .insert_with_ttl(&cache_key, value.clone(), std::time::Duration::from_secs(#ttl))
                            .await
                        {
                            tracing::warn!(%err, "failed to set cache");
                        }
                        Ok(value)
                    }

                    pub async fn #clean_cached_name(&self) -> ApplicationResult<()> {
                        let cache_key = format!("{}:{}",cache_kit::CACHE_PREFIX, #prefix);
                        self.cache.delete_prefix(&cache_key).await?;
                        Ok(())
                    }
                });
            }
        }
    }

    let gen_cached_impl = if !cached_methods.is_empty() {
        quote! {
            use cache_kit::CacheTrait as _;
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

fn parse_cached_args(attr: &Attribute) -> CachedArgs {
    let mut args = CachedArgs {
        prefix: Default::default(),
        ttl: Default::default(),
    };

    if attr.meta.require_list().is_err() {
        panic!("#[cached] requires parameters like #[cached(prefix = \"...\", ttl = \"...\")]");
    }

    attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("prefix") {
            // prefix = "xxx"
            let lit: Lit = meta.value()?.parse()?;
            if let Lit::Str(s) = lit {
                args.prefix = s.value();
            } else {
                return Err(meta.error("prefix must be a string literal"));
            }
        } else if meta.path.is_ident("ttl") {
            // ttl = "10min"
            let lit: Lit = meta.value()?.parse()?;
            if let Lit::Str(s) = lit {
                args.ttl = parse_duration(&s.value()).expect("ttl must be a humantime");
            } else {
                return Err(meta.error("ttl must be a string literal"));
            }
        } else {
            return Err(meta.error("unknown attribute parameter"));
        }
        Ok(())
    })
    .expect("invalid #[cached(...)] syntax");

    args
}
