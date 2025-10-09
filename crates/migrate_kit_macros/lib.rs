use proc_macro::TokenStream;
use quote::quote;
use std::{env, fs};

#[proc_macro]
pub fn embed_dir(input: TokenStream) -> TokenStream {
    let dir = input.to_string().trim_matches('"').to_string();
    let dir = expand_env_vars(&dir);
    let mut entries = vec![];
    let mut files = vec![];

    for entry in fs::read_dir(&dir).expect("Failed to read dir") {
        let entry = entry.expect("Failed to read dir entry");
        let path = entry.path();
        if path.is_file() {
            files.push(path);
        }
        files.sort_by(|a, b| a.cmp(&b));
    }

    for path in files {
        let filename = path
            .file_stem()
            .expect("Failed to get file stem")
            .to_str()
            .expect("Failed to convert to str")
            .to_string();
        let content = fs::read_to_string(&path).expect("Failed to read file");
        let checksum = blake3::hash(content.as_bytes()).to_hex().to_string();
        entries.push(quote! {
            migrate_kit::Migration {
                version: #filename,
                content: #content,
                checksum: #checksum,
            }
        });
    }
    let tokens = quote! {
        &[ #(#entries),* ]
    };
    tokens.into()
}

fn expand_env_vars(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '$' {
            // 处理 $VAR 或 ${VAR}
            if let Some('{') = chars.peek() {
                chars.next(); // 跳过 {
                let mut var_name = String::new();
                while let Some(&nc) = chars.peek() {
                    if nc == '}' {
                        chars.next(); // 跳过 }
                        break;
                    }
                    var_name.push(nc);
                    chars.next();
                }
                if let Ok(val) = env::var(&var_name) {
                    result.push_str(&val);
                }
            } else {
                // 普通 $VAR_NAME
                let mut var_name = String::new();
                while let Some(&nc) = chars.peek() {
                    if !nc.is_alphanumeric() && nc != '_' {
                        break;
                    }
                    var_name.push(nc);
                    chars.next();
                }
                if let Ok(val) = env::var(&var_name) {
                    result.push_str(&val);
                }
            }
        } else {
            result.push(c);
        }
    }
    result
}
