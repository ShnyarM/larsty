fn main() {
    frontend_generator::lex::pre_compile_lexer();
    frontend_generator::parse::pre_compile_parser();

    println!("cargo:rerun-if-changed=../frontend_generator/src/lex/rules.rs");
    println!("cargo:rerun-if-changed=../frontend_generator/src/parse/rules.rs");
}
