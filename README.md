# LALR(1) Parser Generator

A rust program that, given the grammar of a language, is able to generate a LALR(1) parser for that grammar at compile time,
which can then be used at runtime. Supports operator precedence and associativity rules to automatically resolve shift and 
reduce conflicts.

Currently, the grammar defined in the code is a programming-language-like grammar. The generator is able to create a parser that
can successfully create a usable AST from the following snippet of code:

```
i32 myfunction() {
  let x = 5;
  if(6){
    let y = 6 + 8;
  }else if(7){
    z=6;
  }else {
    t=8;
  }
  return 7;
}

void myotherfunc(){
  let foo = 7;
}
```

The code currently does not support much more than shown in the example, but the grammar can easily be extended to parse
a more complex language.

## Lexer

This parser generator also includes a lexer, that can turn a string into a stream of tokens. The parser operates on these tokens, so
the grammar has to be defined using these tokens.
Lexer rules are specified using regular expressions. Using these rules, a
non-deterministic FSM is generated, which is then converted into a reduced deterministic FSM.

The tokens are defined in [this file](frontend/src/token.rs). Adding tokens can be done by expanding the enum type and creating an index for the
token.

Lexer rules are defined in [this file](frontend/src/lex/rules.rs). The deterministic DFA will be created using the rules specified in the file.

## Parser

The grammar is specified fully in [this file](frontend/src/parse/rules.rs), through a sequence of rules. Each rule
specifies:

1. What Non-terminal can be derived from this rule
2. The actual rule, as a sequence of Non-terminals and tokens
3. A generator, which specifies how the sequence of tokens and non-terminals should be converted into an AST Node. The input 
non-terminals are also AST nodes.

The final output after parsing is one AST Node of type `Program`. The Node type is specified [here](frontend/src/ast.rs).

Additionally, tokens can have precedence and associativity rules associated with them. These are also specified in [the same file](frontend/src/parse/rules.rs). This is mainly used to break conflicts with binary operators.

## Parser generation

When compiling the program, both the lexer and parser get generated, and their structure get included into the final binary executable of the `core` program.
The `core` program takes in an input file, and prints the AST of the parsed input. Since the parser and lexer get created at compile,
the parser and lexer do not have to be derived again, but rather only extracted and executed.

## TO DOs

- Create a cleaner interface or defining the lexer and grammar rules
- Make the implementation more generic, and less reliant on pre-defined types
- (If possible) Create a generic package, that can be used for other programs
