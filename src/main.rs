mod lexer;
use lexer::{Lexer, Token};

fn main() {
    let mut lex = Lexer::new("def x = 'adfgasdgasdg'".to_string());
    let mut toks = vec![];
    loop {
        let (_, result)= lex.next_token();
        if result == Token::EOF { break }
        toks.push(result);
    }
    dbg!(toks);
}
