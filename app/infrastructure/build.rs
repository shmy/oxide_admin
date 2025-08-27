fn main() {
    println!("cargo:rerun-if-changed=migration/sql");
}
