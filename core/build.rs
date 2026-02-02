fn main() {
    frontend_generator::lex::pre_compile_lexer();

    println!("cargo:rerun-if-changed=../frontend_generator/src/lex/rules.rs");
}
