fn main() {
    frontend::lex::pre_compile_lexer();
    frontend::parse::pre_compile_parser();

    println!("cargo:rerun-if-changed=../frontend/src/lex/rules.rs");
    println!("cargo:rerun-if-changed=../frontend/src/parse/rules.rs");
}
