mod lexer;
mod ast;

use lexer::{Tokenizer, Token};

// 1: tokenize into basic symbols, keywords, and literals
// 2: lex into language specific tokens
// 3: parse into abstract syntax tree

fn main() {
    if let Ok(input) = std::fs::read_to_string("test_file.kat") {
        // 1: tokenize into basic symbols, keywords, and literals
        let mut tokenizer = Tokenizer::new(input);
        let mut tokens = vec![];
        loop {
            let (_, result)= tokenizer.next_token();
            if result == Token::EOF { break }
            tokens.push(result);
        }
        dbg!(tokens);

        // 2: lex into language specific tokens
    } else {
        println!("can't find file or something")
    }
}
