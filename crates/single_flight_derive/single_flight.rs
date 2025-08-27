use cruet::Inflector as _;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{ItemFn, parse_macro_input};

pub fn generate(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    let vis = &input.vis;
    let sig = &input.sig;
    let block = &input.block;
    let fn_name = &sig.ident;

    // 模块作用域 static 名称
    let static_ident = format_ident!(
        "SINGLE_FLIGHT_{}",
        fn_name.to_string().to_screaming_snake_case()
    );

    // 参数列表用于生成 key
    let args_idents: Vec<_> = sig
        .inputs
        .iter()
        .filter_map(|arg| match arg {
            syn::FnArg::Typed(pat_type) => Some(&*pat_type.pat),
            _ => None,
        })
        .collect();

    // key 表达式
    let key_expr = if args_idents.is_empty() {
        quote! { () }
    } else if args_idents.len() == 1 {
        let ty = args_idents[0];
        quote! { #ty.clone() }
    } else {
        quote! { (#(#args_idents.clone(),)*) }
    };

    // key 类型
    let key_ty = if args_idents.is_empty() {
        quote! { () }
    } else {
        let types: Vec<_> = sig
            .inputs
            .iter()
            .filter_map(|arg| match arg {
                syn::FnArg::Typed(pat_type) => Some(&*pat_type.ty),
                _ => None,
            })
            .collect();
        if types.len() == 1 {
            let ty = types[0];
            quote! { #ty }
        } else {
            quote! { (#(#types),*) }
        }
    };

    // 返回类型
    let output_ty = match &sig.output {
        syn::ReturnType::Type(_, ty) => quote! { #ty },
        _ => panic!("async fn must have a return type"),
    };

    // 生成宏展开内容
    let expanded = quote! {
        #vis #sig where #output_ty: Clone {
            // 函数作用域 static
            static #static_ident: std::sync::LazyLock<singleflight_async::SingleFlight<#key_ty, #output_ty>> =
                           std::sync::LazyLock::new(singleflight_async::SingleFlight::new);

            let key = #key_expr;
            #static_ident.work(key, || async move #block).await
        }
    };

    TokenStream::from(expanded)
}
