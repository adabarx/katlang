#![allow(dead_code)]
use regex_macro::regex;

#[derive(Debug)]
enum Token {
    Def(usize),
    Delimiter(usize),
    Extern(usize),
    OpenParentheses(usize),
    ClosedParentheses(usize),
    Comma(usize),
    Identifier(usize, String),
    Number(usize, f64),
    Operator(usize, String),
}

impl Token {
    pub fn position(&self) -> &usize {
        match self {
            Self::Def(p) => p,
            Self::Delimiter(p) => p,
            Self::Extern(p) => p,
            Self::OpenParentheses(p) => p,
            Self::ClosedParentheses(p) => p,
            Self::Comma(p) => p,
            Self::Identifier(p, _) => p,
            Self::Number(p, _) => p,
            Self::Operator(p, _) => p,
        }
    }
}

fn next_position(p: Option<usize>) -> Option<usize> {
    if let Some(pos) = p {
        Some(pos + 1)
    } else {
        Some(0)
    }
}

fn lexer(input: &str) -> Vec<Token> {
    let comment_rgx = regex!(r"(?m)#.*\n");
    let preprocessed = comment_rgx.replace_all(input, "\n");

    let token_rgx = regex!(concat!(
        r"(?P<ident>\p{Alphabetic}\w*)|",
        r"(?P<number>\d+\.?\d*)|",
        r"(?P<delimiter>;)|",
        r"(?P<oppar>\()|",
        r"(?P<clpar>\))|",
        r"(?P<comma>,)|",
        r"(?P<operator>\S)"
    ));

    let mut position = None;

    token_rgx
        .captures_iter(preprocessed.to_string().as_str())
        .map(|capture| -> Token {
            position = next_position(position);
            let current_pos = position.unwrap();

            if let Some(lexeme) = capture.name("ident") {
                match lexeme.as_str() {
                    "def" => Token::Def(current_pos),
                    "extern" => Token::Extern(current_pos),
                    ident => Token::Identifier(current_pos, ident.to_string()),
                }
            } else if let Some(lexeme) = capture.name("number") {
                Token::Number(
                    current_pos,
                    lexeme.as_str().parse::<f64>()
                        .expect("lexer failed parsing a number")
                )
            } else if capture.name("delimiter").is_some() {
                Token::Delimiter(current_pos)
            } else if capture.name("oppar").is_some() {
                Token::OpenParentheses(current_pos)
            } else if capture.name("clpar").is_some() {
                Token::ClosedParentheses(current_pos)
            } else if capture.name("comma").is_some() {
                Token::Comma(current_pos)
            } else if let Some(lexeme) = capture.name("operator") {
                Token::Operator(current_pos, lexeme.as_str().to_string())
            } else {
                panic!("syntax error")
            }
        })
        .collect()
}

fn main() {
    dbg!(lexer("def x = 1 + 2"));
}
