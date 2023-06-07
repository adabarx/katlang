#![allow(dead_code)]
use anyhow::{Result, bail};

// TODO: tokenize triangles, pipes, eq, and other multi-char operators
// TODO: handle comments

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    // user generated
    Ident(String),
    Lit(LiteralToken),

    // keywords
    Import,
    Let,
    Mut,
    Def,
    Impl,
    Struct,
    Enum,
    Object,
    Trait,
    Desc,
    If,
    Elif,
    Else,
    Match,
    True,
    False,
    And,
    Or,
    Xor,
    Not,

    // multi-char operators
    ReturnOp,
    PipeOp,
    AddAssignOp,
    SubtractAssignOp,
    MultiplyAssignOp,
    DivideAssignOp,
    ModulusAssignOp,
    AndOp,
    OrOp,
    XorOp,
    EqualOp,
    NotEqualOp,
    LessThanOp,
    GrtrThanOp,
    LessEqualOp,
    GrtrEqualOp,
    // UnsafeEqualOp,
    // UnsafeNotEqualOp,
    // UnsafeLessThanOp,
    // UnsafeGrtrThanOp,
    // UnsafeLessEqualOp,
    // UnsafeGrtrEqualOp,

    // newlines/whitespace
    NewLine,
    Tab,

    // surrounding chars
    LParen,
    RParen,
    LSquirly,
    RSquirly,
    LBrack,
    RBrack,
    LTriangle,
    RTriangle,
    LAngle,
    RAngle,

    // symbols
    Comma,
    Dot,
    Pipe,
    Plus,
    Dash,
    Equal,
    FSlash,
    BSlash,
    Colon,
    SemiColon,
    Bang,
    At,
    Octothorpe,
    Dollar,
    Percent,
    Caret,
    Ampersand,
    Asterisk,
    Question,
    Tilde,
    Grave,

    EOF,
}

#[derive(Clone, Debug, PartialEq)]
pub enum LiteralToken {
    Str(String),
    Num(f64),
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
            b'{'  => Token::LSquirly,
            b'}'  => Token::RSquirly,
            b'('  => Token::LParen,
            b')'  => Token::RParen,
            b'['  => Token::LBrack,
            b']'  => Token::RBrack,
            b'<'  => match self.next_char()? {
                b'|' => Token::RTriangle,
                b'=' => Token::LessEqualOp,
                _ => Token::LAngle,
            },
            b'>'  => match self.next_char()? {
                b'=' => Token::GrtrEqualOp,
                _ => Token::RAngle,
            }
            b','  => Token::Comma,
            b'.'  => Token::Dot,
            b'?'  => Token::Question,
            b':'  => Token::Colon,
            b';'  => Token::SemiColon,
            b'!'  => if self.next_char()? == b'=' {Token::NotEqualOp } else { Token::Bang },
            b'@'  => Token::At,
            b'#'  => Token::Octothorpe,
            b'$'  => Token::Dollar,
            b'%'  => Token::Percent,
            b'^'  => if self.next_char()? == b'&' { Token::XorOp } else { Token::Caret },
            b'&'  => if self.next_char()? == b'&' { Token::AndOp } else { Token::Ampersand },
            b'*'  => if self.next_char()? == b'=' { Token::MultiplyAssignOp } else { Token::Asterisk },
            b'-'  => if self.next_char()? == b'=' { Token::SubtractAssignOp } else { Token::Dash },
            b'='  => if self.next_char()? == b'=' { Token::EqualOp } else { Token::Equal },
            b'+'  => if self.next_char()? == b'=' { Token::AddAssignOp } else { Token::Plus },
            b'|'  => match self.next_char()? {
                b'>' => Token::LTriangle,
                b'|' => Token::OrOp,
                _ => Token::Pipe,
            },
            b'\\' => Token::BSlash,
            b'/'  => match self.next_char()? {
                b'/' => loop { if self.next_char()? != b' ' { break Token::NewLine } } // handle comments
                b'=' => Token::DivideAssignOp,
                _ => Token::FSlash,
            },
            b'~'  => Token::Tilde,
            b'`'  => Token::Grave,
            b'\t' => Token::Tab,
            b'\n' => Token::NewLine,

            b'\'' | b'"' => Token::Lit(LiteralToken::Str(self.read_string_literal()?.to_string())),
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => {
                let ident = self.read_ident()?;
                match ident.as_str() {
                    "import" => Token::Import,
                    "let"    => Token::Let,
                    "mut"    => Token::Mut,
                    "def"    => Token::Def,
                    "struct" => Token::Struct,
                    "enum"   => Token::Enum,
                    "object" => Token::Object,
                    "trait"  => Token::Trait,
                    "desc"   => Token::Desc,
                    "impl"   => Token::Impl,
                    "if"     => Token::If,
                    "elif"   => Token::Elif,
                    "else"   => Token::Else,
                    "match"  => Token::Match,
                    "true"   => Token::True,
                    "false"  => Token::False,
                    "and"    => Token::And,
                    "or"     => Token::Or,
                    "xor"    => Token::Xor,
                    "not"    => Token::Not,
                    _ => Token::Ident(ident.to_string())
                }
            },
            b'0'..=b'9' => {
                Token::Lit(LiteralToken::Num(self.read_number_literal()?))
            },
            0 => Token::EOF,
            _ => Token::EOF,
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

    fn peek(&self, spaces: usize) -> Result<u8> {
        let peek_pos = self.position + spaces;
        if peek_pos >= self.input.len() { bail!("EOF") }

        Ok(self.input[peek_pos])
    }
    
    fn peek_match(&self, input: &str) -> Result<bool> {
        if input.chars().count() + self.position + 1 >= self.input.len() { bail!("EOF") };

        let mut forward = 1;
        for ch in input.chars() {
            if self.input[self.position + forward] != ch as u8 { return Ok(false) }
            else { forward += 1 }
        }
        Ok(true)
    }

    fn prev_match(&self, input: u8) -> bool {
        if self.input[self.position - 1] == input { true } else { false }
    }
}

