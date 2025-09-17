#[cfg(debug_assertions)]
fn main() {}

#[cfg(not(debug_assertions))]
fn main() {
    use std::{fs, path::Path};

    const CONTENT_ENCODING_EXTENSION: &str = include_str!("../../frontend/dist/.EXTENSION");
    let index_html_data = fs::read(&format!(
        "../../frontend/dist/index.html.{}",
        CONTENT_ENCODING_EXTENSION
    ))
    .unwrap();
    let sign_in_html_data = fs::read(&format!(
        "../../frontend/dist/sign_in.html.{}",
        CONTENT_ENCODING_EXTENSION
    ))
    .unwrap();

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir).join("data.rs");

    fs::write(
        out_path,
        format!(
            r#"
const INDEX_HTML_DATA: &[u8] = &{:?};
const SIGN_IN_HTML_DATA: &[u8] = &{:?};
        "#,
            index_html_data, sign_in_html_data
        ),
    )
    .unwrap();
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../../frontend/dist");
}
