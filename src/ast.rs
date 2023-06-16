#![allow(dead_code, unused_variables)]
use anyhow::{Result, bail, anyhow};

use crate::lexer::{Token, UnaryOperator, Literal, BinaryOperator};

type Statements = Vec<ASTNode>;

#[derive(Clone, Debug, PartialEq)]
pub enum ASTNode {
    Expr(Expression),
    Decl(Declaration),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    Unary {
        expression: Box<Expression>,
        operator: UnaryOperator,
    },
    Binary {
        lhs: Box<Expression>,
        operator: BinaryOperator,
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

    pub fn parse_statement(&mut self) -> Result<ASTNode> {
        use Token::*;

        while self.token == NewLine { self.next_token()?; }

        let rv = match &self.token {
            Ident(_) | Lit(_) | UnaryOp(_) => ASTNode::Expr(self.parse_expression(true)?),
            Let =>
                if let Ident(name) = self.next_token()? {
                    if self.next_token()? == Assign {
                        self.next_token()?;
                        ASTNode::Decl(Declaration::Variable {
                            name,
                            expression: self.parse_expression(true)?
                        })
                    } else {
                        bail!("invalid let statement")
                    }
                } else {
                    bail!("invalid let statement")
                }
            EOF => bail!("EOF"),
            _ => bail!("syntax error: parse_statement")
        };

        Ok(rv)
    }

    fn parse_expression(&mut self, check_binary: bool) -> Result<Expression> {
        use Token::*;
        let rv = match &self.token {
            Ident(id) => Expression::Variable(id.clone()),
            Lit(lit) => Expression::Literal(lit.clone()),
            UnaryOp(operator) => Expression::Unary {
                operator: operator.clone(),
                expression: Box::new(self.next_value()?)
            },
            _ => bail!("invalid expression")
        };

        Ok(match self.next_token() {
            Ok(BinaryOp(operator)) =>
                if check_binary {
                    self.next_token()?;
                    self.parse_binary_expression(operator, rv)?
                } else {
                    rv
                },
            Ok(NewLine) => rv,
            Err(_) => {
                self.token = EOF;
                rv
            },
            _ => bail!("invalid token after expression: {:?}", &self.token)
        })
    }

    fn next_value(&mut self) -> Result<Expression> {
        use Token::*;
        Ok(match self.next_token()? {
            Ident(id) => Expression::Variable(id),
            Lit(lit) => Expression::Literal(lit),
            _ => bail!("invalid value")
        })
    }

    fn parse_binary_expression(&mut self, operator: BinaryOperator, lhs: Expression) -> Result<Expression> {
        let rv = Expression::Binary {
            operator,
            lhs: Box::new(lhs),
            rhs: Box::new(self.parse_expression(false)?),
        };
        Ok(match self.token.clone() {
            Token::BinaryOp(operator) => {
                self.next_token()?; // on start of next expression
                self.parse_binary_expression(operator, rv)?
            },
            Token::NewLine | Token::EOF=> rv,
            _ => bail!("invalid token after binary expression: {:?}", &self.token)
        })
    }
}

