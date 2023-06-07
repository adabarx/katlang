mod lexer;
mod ast;

use lexer::{Lexer, Token};

// 1: tokenize into basic symbols, keywords, and literals
// 2: parse into abstract syntax tree

fn main() {
    if let Ok(input) = std::fs::read_to_string("test_file.kat") {
        // 1: tokenize into basic symbols, keywords, and literals
        let mut lexer = Lexer::new(input);
        let mut tokens = vec![];
        loop {
            match lexer.next_token() {
                Ok((_, result)) =>
                    if result == Token::EOF { break } 
                    else { tokens.push(result) },
                Err(e) => {
                    dbg!(e);
                    break;
                }
            }
        }
        dbg!(&tokens);

        // 2: parse into abstract syntax tree
    } else {
        println!("can't find file or something")
    }
}
