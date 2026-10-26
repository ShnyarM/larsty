#[cfg(test)]
mod test{
    use super::regexp::Regexp;

    #[test]
    fn lex_hello_world() {
        let regexp = Regexp::string("Hello World!"); 
        let dfa = regexp.to_dfa();

        assert!(dfa.execute("Hello World!"));
        assert!(!dfa.execute("faasdfHello World!"));
        assert!(!dfa.execute("Hello World!a"));
        assert!(!dfa.execute(""));
    }

    #[test]
    fn lex_empty() {
        let dfa = Regexp::string("").to_dfa();

        assert!(dfa.execute(""));
        assert!(!dfa.execute("Hello"));

        let dfa = Regexp::empty().to_dfa();
        assert!(dfa.execute(""));
        assert!(!dfa.execute("Hello"));
    }

    #[test]
    fn lex_or() {
        let dfa = Regexp::or(
            Regexp::string("This is one."),
            Regexp::string("Another one..")
        ).to_dfa();

        assert!(dfa.execute("This is one."));
        assert!(dfa.execute("Another one.."));
        assert!(!dfa.execute("Hello"));
        assert!(!dfa.execute("This is different."));
        assert!(!dfa.execute(""));
    }

    #[test]
    fn lex_concat() {
        let dfa = Regexp::concat(
            Regexp::string("This is one."),
            Regexp::string("Another one..")
        ).to_dfa();

        assert!(dfa.execute("This is one.Another one.."));
        assert!(!dfa.execute("This is one."));
        assert!(!dfa.execute("Another one.."));
        assert!(!dfa.execute("Hello"));
        assert!(!dfa.execute("This is different."));
        assert!(!dfa.execute(""));
    }

    #[test]
    fn lex_rep() {
        let dfa = Regexp::rep(
            Regexp::string("repetition")
        ).to_dfa();

        assert!(dfa.execute(""));
        assert!(dfa.execute("repetition"));
        assert!(dfa.execute("repetitionrepetition"));
        assert!(dfa.execute("repetitionrepetitionrepetition"));
        assert!(!dfa.execute("repetitionrepetitionrepe"));
        assert!(!dfa.execute("repetonrepetition"));
        assert!(!dfa.execute("Hello"));
        assert!(!dfa.execute("This is different."));
    }

    #[test]
    fn lex_complex_expression() {
        // ((ab | cd) e*) f
        let regex = Regexp::concat(
            Regexp::concat(
                Regexp::or(
                    Regexp::string("ab"),
                    Regexp::string("cd"),
                ),
                Regexp::rep(
                    Regexp::string("e"),
                ),
            ),
            Regexp::string("f"),
        );

        let dfa = regex.to_dfa();

        // accepted
        assert!(dfa.execute("abf"));
        assert!(dfa.execute("abef"));
        assert!(dfa.execute("abeeef"));
        assert!(dfa.execute("cdf"));
        assert!(dfa.execute("cdef"));
        assert!(dfa.execute("cdeeeeeef"));

        // rejected
        assert!(!dfa.execute("ab"));
        assert!(!dfa.execute("cd"));
        assert!(!dfa.execute("abefg"));
        assert!(!dfa.execute("ef"));
        assert!(!dfa.execute("abeee"));
        assert!(!dfa.execute("hello"));
    }
}
