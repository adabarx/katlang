use clap::{command, Arg};

mod lexer;
mod ast;

use lexer::Lexer;
use ast::Parser;


// 1: tokenize into basic symbols, keywords, and literals
// 2: parse into abstract syntax tree

fn main() {
    let result = command!().arg(Arg::new("filepath").required(true))
        .get_matches();
    let file_path = result.get_one::<String>("filepath").unwrap();
    if let Ok(input) = std::fs::read_to_string(file_path) {
        // 1: tokenize into basic symbols, keywords, and literals
        let mut lexer = Lexer::new(input);
        let mut tokens = vec![];
        loop {
            match lexer.next_token() {
                Ok((_, result)) => tokens.push(result),
                Err(e) => { dbg!(e); break; }
            }
        }
        dbg!(&tokens);

        let parser = Parser::new(tokens);
        let astree = parser.lfg();

        dbg!(&astree);

        // 2: parse into abstract syntax tree
    } else {
        println!("can't find file or something")
    }
}
