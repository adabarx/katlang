#![allow(dead_code, unused_variables)]
use anyhow::{Result, bail, anyhow};

use crate::lexer::{Token, UnaryOperator, Literal, BinaryOperator, Surround, Scope};

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
    LexicalScope(Vec<ASTNode>),
    BlockScope(Vec<ASTNode>),
}

impl Expression {
    pub fn from_scope(scope: Scope, statments: Vec<ASTNode>) -> Self {
        match scope {
            Scope::Block => Self::BlockScope(statments),
            Scope::Lexical => Self::LexicalScope(statments),
            _ => todo!(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Declaration {
    Variable {
        name: String,
        expression: Expression,
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ScopeTree {
    Root(Vec<ASTNode>),
    Lexical(Vec<ASTNode>),
    Block(Vec<ASTNode>),
}

impl ScopeTree {
    pub fn new(scope_type: Scope) -> Self {
        match scope_type {
            Scope::Block => Self::Block(vec![]),
            Scope::Lexical => Self::Lexical(vec![]),
            _ => todo!("only block and lex")
        }
    }

    pub fn statements(&self) -> Vec<ASTNode> {
        match self {
            Self::Block(statements) => statements.clone(),
            Self::Lexical(statements) => statements.clone(),
            Self::Root(statements) => statements.clone(),
        }
    }

    pub fn push(&mut self, node: ASTNode) {
        match self {
            Self::Root(statements) => statements.push(node),
            Self::Lexical(statements) => statements.push(node),
            Self::Block(statements) => statements.push(node),
        }
    }

    pub fn is_scope(&self, compare: Scope) -> bool {
        match self {
            Self::Block(_) if matches!(compare, Scope::Block) => true,
            Self::Lexical(_) if matches!(compare, Scope::Lexical) => true,
            _ => false,
        }
    }
}

impl Default for ScopeTree {
    fn default() -> Self {
        Self::Root(vec![]) 
    }
}

pub struct Parser {
    position: usize,
    length: usize,
    input: Vec<Token>,
    token: Token,
    scope_stack: Vec<ScopeTree>,
}

impl Parser {
    pub fn new(input: Vec<Token>) -> Self {
        Self {
            position: 0,
            length: input.iter().len(),
            token: input[0].clone(),
            input,
            scope_stack: vec![ScopeTree::default()],
        }
    }

    pub fn lfg(mut self) -> Vec<ScopeTree> {
        loop {
            match self.parse_statement() {
                Ok(node) => {
                    self.scope_stack.last_mut().unwrap().push(node);
                    dbg!(&self.scope_stack);
                },
                Err(e) => { dbg!(e); break },
            }
        }
        self.scope_stack
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
        dbg!(&self.token);
        let rv = match self.token {
            Ident(_) | Lit(_) | BinaryOp(_) |UnaryOp(_) =>
                ASTNode::Expr(self.parse_expression(true)?),
            Surr(Surround::Open(new_scope_tok)) => {
                self.scope_stack.push(ScopeTree::new(new_scope_tok));
                self.next_token()?;
                return self.parse_statement();
            },
            Surr(Surround::Close(exit_scope_tok)) => {
                println!("closing scope statement");
                let closing_scope = self.scope_stack.pop()
                    .ok_or(anyhow!("no more scope???"))?;
                if closing_scope.is_scope(exit_scope_tok) {
                    dbg!(&self.next_token()?);
                    return Ok(
                        ASTNode::Expr(
                            Expression::from_scope(exit_scope_tok, closing_scope.statements())
                        )
                    );
                } else {
                    panic!("mismatched closing character")
                }
            },
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

        if let Surr(surround) = self.token {
            match surround {
                Surround::Open(new_scope_tok) => {
                    self.scope_stack.push(ScopeTree::new(new_scope_tok));
                    self.next_token()?;
                },
                Surround::Close(exit_scope_tok) => {
                println!("closing scope expression");
                    let closing_scope = self.scope_stack.pop()
                        .ok_or(anyhow!("no more scope???"))?;
                    self.next_token()?;
                    if closing_scope.is_scope(exit_scope_tok) {
                        return Ok(Expression::from_scope(exit_scope_tok, closing_scope.statements()));
                    }
                },
            }
        }

        // skip newlines
        while let NewLine = &self.token { self.next_token()?; }

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
            Token::NewLine | Token::EOF => rv,
            _ => bail!("invalid token after binary expression: {:?}", &self.token)
        })
    }
}

