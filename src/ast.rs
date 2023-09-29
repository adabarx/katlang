#![allow(dead_code, unused_variables)]
use anyhow::{Result, bail, anyhow};

use crate::lexer::{Token, Operator, Literal};

type Statements = Vec<ASTNode>;

#[derive(Clone, Debug, PartialEq)]
pub enum ASTNode {
    Expr(Expression),
    Decl(Declaration),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    Unary {
        operator: Operator,
        expression: Box<Expression>,
    },
    Binary {
        operator: Operator,
        lhs: Box<Expression>,
        rhs: Box<Expression>,
    },
    Literal(Literal),
    Variable(String),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Declaration {
    Variable {
        name: String,
        expression: Expression,
    }
}

pub struct Parser {
    position: usize,
    length: usize,
    input: Vec<Token>,
    token: Token,
}

impl Parser {
    pub fn new(input: Vec<Token>) -> Self {
        Self {
            position: 0,
            length: input.iter().len(),
            token: input[0].clone(),
            input,
        }
    }

    fn peek(&self, forward: usize) -> Result<Token> {
        self.input.get(self.position + forward).ok_or(anyhow!("EOF")).cloned()
    }

    fn next_token(&mut self) -> Result<Token> {
        if self.input.get(self.position + 1).is_none() { bail!("EOF") }
        self.position += 1;
        self.token = self.input[self.position].clone();
        Ok(self.token.clone())
    }

    fn parse_statement(&mut self) -> Result<ASTNode> {
        use Token::*;
        let rv = match &self.token {
            Ident(_) | Lit(_) | Op(_) => ASTNode::Expr(self.parse_expression()?),
            Let =>
                if let Ident(name) = self.next_token()? {
                    if self.next_token()? == Assign {
                        self.next_token()?;
                        ASTNode::Decl(Declaration::Variable {
                            name,
                            expression: self.parse_expression()?
                        })
                    } else {
                        bail!("invalid let statement")
                    }
                } else {
                    bail!("invalid let statement")
                }
            _ => bail!("syntax error: parse_statement")
        };
        self.next_token()?;
        Ok(rv)
    }

    fn parse_expression(&mut self) -> Result<Expression> {
        todo!()
    }
}

