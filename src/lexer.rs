#![allow(dead_code)]
use anyhow::{Result, bail};

// TODO: tokenize triangles, pipes, eq, and other multi-char operators
// TODO: handle comments

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Ident(String),
    Lit(Literal),
    Op(Operator),
    Cond(Conditional),
    Surr(Surround),

    Let,
    Assign,
    NewLine,
    Comma,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Operator {
    NotAnd,
    NotOr,
    NotXor,
    And,
    Or,
    Xor,
    Not,
}

impl Operator {
    pub fn is_unary(&self) -> bool {
        match self {
            Self::Not => true,
            _ => false
        }
    }

    pub fn is_binary(&self) -> bool {
        match self {
            Self::Not => false,
            _ => true
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Surround {
    Open(Scope),
    Close(Scope),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Scope {
    Tuple,
    List,
    Block,
    Lexical,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Conditional {
    If,
    Elif,
    Else,
    Match,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Literal {
    True,
    False,
}

#[derive(Debug)]
pub struct Lexer {
    position: usize,
    ch: u8,
    input:Vec<u8>,
}

impl Lexer {
    pub fn new(input: String) -> Self {
        let input = input.into_bytes();
        Self {
            position: 0,
            ch: input[0], // load first char from input string
            input,
        }
    }

    fn next_char(&mut self) -> Result<u8>{
        if self.position + 1 >= self.input.len() { bail!("EOF") } 

        self.position += 1;
        self.ch = self.input[self.position].clone();
        Ok(self.ch.clone())
    }

    pub fn next_token(&mut self) -> Result<(usize, Token)> {
        while self.ch == b' ' || self.ch == b'\t' { // skip whitespace
            self.next_char()?;
        }

        let tok = match self.ch {
            b','  => Token::Comma,
            b'='  => Token::Assign,
            b'\n' => Token::NewLine,

            b'{'  => Token::Surr(Surround::Open(Scope::Block)),
            b'}'  => Token::Surr(Surround::Close(Scope::Block)),
            b'('  => Token::Surr(Surround::Open(Scope::Tuple)),
            b')'  => Token::Surr(Surround::Close(Scope::Tuple)),

            b'!'  => 
                if      self.next_match("and")? { Token::Op(Operator::NotAnd) }
                else if self.next_match("or")?  { Token::Op(Operator::NotOr) }
                else if self.next_match("xor")? { Token::Op(Operator::NotXor) }
                else                            { Token::Op(Operator::Not) },

            b'a'..=b'z' | b'A'..=b'Z' | b'_' => {
                let ident = self.read_ident()?;
                match ident.as_str() {
                    "let"   => Token::Let,
                    "if"    => Token::Cond(Conditional::If),
                    "elif"  => Token::Cond(Conditional::Elif),
                    "else"  => Token::Cond(Conditional::Else),
                    "match" => Token::Cond(Conditional::Match),
                    "true"  => Token::Lit(Literal::True),
                    "false" => Token::Lit(Literal::False),
                    "and"   => Token::Op(Operator::And),
                    "or"    => Token::Op(Operator::Or),
                    "xor"   => Token::Op(Operator::Xor),
                    _ => Token::Ident(ident.to_string())
                }
            },
            // b'0'..=b'9' => {
            //     Token::Lit(LiteralToken::Num(self.read_number_literal()?))
            // },
            0 => bail!("EOF"),
            _ => bail!("Invalid Token"),
        };

         self.next_char()?;

        Ok((self.position, tok))
    }

    fn read_number_literal(&mut self) -> Result<f64> { 
        let pos = self.position;
        let mut decimal = false;
        loop {
            match self.next_char()? {
                b'0'..=b'9' => continue,
                b'.' => if decimal { break } else {
                    decimal = true;
                },
                _ => break,
            }
        }
        Ok(String::from_utf8_lossy(&self.input[pos..self.position])
            .to_string()
            .parse::<f64>()
            .unwrap())
    }

    fn read_string_literal(&mut self) -> Result<String> { 
        // TODO: escape quote \"
        let quote = self.ch; // store the current single/double quote
        let mut s = vec![];
        loop {
            let ch = self.next_char()?;
            if ch == quote { break }
            s.push(ch as char)
        }
        Ok(String::from_iter(s.iter()))

    }

    fn read_ident(&mut self) -> Result<String> {
        let pos = self.position;
        while self.ch.is_ascii_alphabetic() || self.ch == b'_' {
            self.next_char()?;
        }
        Ok(String::from_utf8_lossy(&self.input[pos..self.position]).to_string())
    }

    fn peek(&self, offset: usize) -> Result<u8> {
        let peek_pos = self.position + offset;
        if peek_pos >= self.input.len() { bail!("EOF") }

        Ok(self.input[peek_pos])
    }
    
    fn next_match(&self, input: &str) -> Result<bool> {
        // check if the upcoming characters match the input
        // and advance the lexer if true
        let mut offset = 1;
        for ch in input.chars() {
            if self.peek(offset)? == ch as u8 { offset += 1 }
            else { return Ok(false) }
        }

        for _ in 0..offset { self.next_char()?; }

        Ok(true)
    }

    fn prev_match(&self, input: u8) -> bool {
        if self.input[self.position - 1] == input { true } else { false }
    }
}

