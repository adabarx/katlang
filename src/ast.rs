#![allow(dead_code, unused_variables)]
use anyhow::{Result, bail, anyhow};

use crate::lexer::Token;

type Statements = Vec<ASTNode>;

pub enum ASTNode {
    Expression(Expression),
    Declaration(Declaration),
    TypeExpression(TypeExpression),
    Import,
}

pub enum Expression {
    Unary {
        operator: Operator,
        expression: Box<Expression>
    },
    Binary {
        operator: Operator,
        lhs: Box<Expression>,
        rhs: Box<Expression>,
    },
    FunctionCall {
        callee: String,
        signature: (TypeExpression, TypeExpression),
        arguments: Vec<Expression>
    },
    Literal(Literal),
    Variable(String),
    MemberAccess {
        object: Box<Expression>,
        property: String,
    }
}

pub struct Type {
    name: String,
    enclosed: Option<Box<TypeExpression>>
}

pub enum TypeExpression {
    Unary(Box<Type>),
    Tuple(Vec<Type>),
}


#[derive(Hash)]
pub struct Pattern {
    struct_name: String,
    destructured: Vec<(String, String)>
}

pub enum Scope {
    Open(Vec<ASTNode>),
    Closed(Vec<ASTNode>),
}

pub enum Conditional {
    If {
        condition: Box<Expression>,
        block: Vec<ASTNode>
    },
    Elif {
        condition: Box<Expression>,
        block: Vec<ASTNode>
    },
    Else(Vec<ASTNode>),
    Match {
        matching_expression: Box<Expression>,
        condition_blocks: Vec<(Pattern, Vec<ASTNode>)>
    }
}

pub enum Operator {
    // maffs
    Plus,     // +
    Minus,    // -
    Multiply, // *
    Divide,   // /
    Modulus,  // %

    // logic
    And, // && and
    Or,  // || or
    Xor, // ^^ xor
    Not, // !  not (unary)

    // compare
    Equal,            // ==
    NotEqual,         // !=
    LessThan,         // >
    GreaterThan,      // <
    LessThanEqual,    // >=
    GreaterThanEqual, // <=

    // unsafe compare
    // for types that only have PartialEq
    UnsafeEqual,            // ==!
    UnsafeNotEqual,         // !=!
    UnsafeLessThan,         // <!
    UnsafeGreaterThan,      // >!
    UnsafeLessThanEqual,    // <=!
    UnsafeGreaterThanEqual, // >=!

    // assignment
    Assign,          // =

    AddAssign,       // +=
    SubtractAssign,  // -=
    MultiplyAssign,  // *=
    DivideAssign,    // /=
    ModulusAssign,   // %=

    // language
    MemberAccess,   // obj.prop
    Return,         // fiz;;
    ReturnNull,     // bar?
    Reference,      // &y
    Dereference,    // *y
    AbsoluteValue,  // +z
    LogicalInverse, // -z
    Pipe, // |-> foo(..$.., 45)                              // spreading a tuple/object
          // |-> fab($[2], "cow", $[0]) // using tuple/object elements in specific spots
          // |-> fad($.x, "cow", $.y)  // using mapped object elements in specific spots
          // |-> fap($.propa, 'd', $.propc)   // using struct elements in specific spots
          // |-> fag                    // using a type that matches the function params

    // TODO: bitwise 
    // BitAnd,          // &
    // BitOr,           // |
    // BitXor,          // ^
    // BitNot,          // ~
    // BitLShift,       // <<
    // BitRShift,       // >>
    // BitAndAssign,    // &=
    // BitOrAssign,     // |=
    // BitXorAssign,    // ^=
    // BitNotAssign,    // ~=
    // BitLShiftAssign, // <<=
    // BitRShiftAssign, // >>=
}

pub enum Declaration {
    Function {
        name: String,
        parameters: Vec<(String, Expression)>,
        return_value: TypeExpression,
        scope_type: Scope,
        block: Vec<ASTNode>,
    },
    Variable {
        name: String,
        value: Expression,
    },
    MuttableVariable {
        name: String,
        value: Expression,
    },
    Struct {
        name: String,
        properties: Vec<(String, TypeExpression)>
    },
    Enum {
        name: String,
        variants: Vec<Variant>,
    },
    Object {
        name: String,
        properties: Vec<TypeExpression>
    },
    Impl {
        type_name: Type,
        block: Vec<ASTNode>
    }
}

pub enum Variant {
    Object(Vec<TypeExpression>),
    Struct(Vec<(String, TypeExpression)>)
}

pub enum Literal {
    String(String),
    Integer(usize),
    Float(f64),
    Array(Vec<Expression>),
    Tuple(Vec<Expression>),
    Struct(Vec<(String, Expression)>),
}

pub fn parse_program(input: Vec<Token>) -> Vec<ASTNode> {
    let mut parser = Parser::new(input);
    let mut ast = vec![];
    while let Ok(node) = parser.parse_statement() {
        ast.push(node);
    }
    ast
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
        let rv = match &self.token {
            Token::Import => self.parse_import_statement()?,
            Token::Let    => ASTNode::Declaration(self.parse_variable_declaration()?),
            Token::Struct => ASTNode::Declaration(self.parse_struct_declaration()?),
            Token::Enum   => ASTNode::Declaration(self.parse_enum_declaration()?),
            Token::Object => ASTNode::Declaration(self.parse_object_declaration()?),
            Token::If     => self.parse_if_statement()?,
            Token::Elif   => self.parse_elif_statement()?,
            Token::Match  => self.parse_match_statement()?,
            Token::Def    => 
                if let Token::Ident(ident) = self.next_token()?
                    { ASTNode::Declaration(self.parse_function_declaration()?) }
                else
                    { ASTNode::Expression(self.parse_anonymouse_function()?) },
            Token::Ident(ident) => ASTNode::Expression(
                self.parse_expression(Some(Expression::Variable(ident.clone())))?
            ),
            _ => bail!("syntax error: parse_statement")
        };
        self.next_token()?;
        Ok(rv)
    }

    fn parse_variable_declaration(&mut self) -> Result<Declaration> {
        // let x = val
        // let mut x = val
        Ok(match self.next_token()? {
            Token::Mut => match self.next_token()? {
                Token::Ident(name) => if self.next_token()? == Token::Equal {
                    Declaration::MuttableVariable {
                        name,
                        value: self.parse_expression(None)?
                    }
                } else {
                    todo!("implement else")
                }
                _ => bail!("syntax error: parse_variable_declaration")
            },
            Token::Ident(name) => if self.next_token()? == Token::Equal {
                Declaration::Variable {
                    name,
                    value: self.parse_expression(None)?
                }
            } else {
                todo!("implement else")
            },
            _ => bail!("syntax error: parse_variable_declaration")
        })
    }

    fn parse_expression(&mut self, expression: Option<Expression>) -> Result<Expression> {
        if self.peek(1)? == Token::NewLine {
            return expression.ok_or(anyhow!("no expression!!!"))
        }

        let rv = Ok(match expression {
            Some(Expression::Variable(ident)) => todo!("impl variable"),
            Some(Expression::Unary { operator, expression }) => todo!("impl unary"),
            Some(Expression::Binary { operator, lhs, rhs }) => todo!("impl binary"),
            Some(Expression::Literal(lit)) => todo!("impl literal"),
            Some(Expression::FunctionCall { callee, signature, arguments }) => todo!("impl function call"),
            Some(Expression::MemberAccess { object, property }) => todo!("impl member access"),
            None => {
                match self.next_token()? {
                    Token::LParen => self.parse_tuple()?,
                    Token::Bang =>
                        Expression::Unary {
                            operator: Operator::Not,
                            expression: Box::new(self.parse_expression(None)?)
                        },
                    Token::Ampersand =>
                        Expression::Unary {
                            operator: Operator::Reference,
                            expression: Box::new(self.parse_expression(None)?)
                        },
                    Token::Asterisk =>
                        Expression::Unary {
                            operator: Operator::Dereference,
                            expression: Box::new(self.parse_expression(None)?)
                        },
                    Token::Plus =>
                        Expression::Unary {
                            operator: Operator::AbsoluteValue,
                            expression: Box::new(self.parse_expression(None)?)
                        },
                    Token::Dash =>
                        Expression::Unary {
                            operator: Operator::LogicalInverse,
                            expression: Box::new(self.parse_expression(None)?)
                        },
                    Token::Ident(ident) => self.match_token_ident(ident)?,
                    _ => bail!("syntax error: parse_variable_declaration")
                }
            }
        });

        if self.peek(1)? == Token::NewLine { rv }
        else { self.parse_expression(Some(rv?)) }
    }

    fn match_token_ident(&mut self, ident: String) -> Result<Expression> {
        Ok(match self.next_token()? {
            Token::Ident(param) =>
                Expression::FunctionCall {
                    callee: ident,
                    signature: (
                        TypeExpression::Unary(Box::new(Type { name: "i32".to_string(), enclosed: None })),
                        TypeExpression::Unary(Box::new(Type { name: "i32".to_string(), enclosed: None }))
                    ),
                    arguments: vec![Expression::Variable(param)]
                },
            Token::SemiColon => 
                if self.next_token()? == Token::SemiColon {
                    Expression::Unary {
                        operator: Operator::ReturnNull,
                        expression: Box::new(Expression::Variable(ident))
                    }
                } else {
                    bail!("syntax error: double semicolon")
                },
            Token::Question =>
                Expression::Unary {
                    operator: Operator::ReturnNull,
                    expression: Box::new(Expression::Variable(ident))
                },
            Token::Plus => todo!("binary add"),
            Token::Dash => todo!("binary subtract"),
            Token::Asterisk => todo!("binary multiply"),
            Token::FSlash => todo!("binary divide"),
            Token::Percent => todo!("binary modulus"),
            Token::Bang => todo!("binary not equal compare"),
            Token::Equal => todo!("binary equal compare"),
            Token::LAngle => todo!("binary less/less-equal than"),
            Token::RAngle => todo!("binary greater/greater-equal than"),
            Token::And => todo!("binary and"),
            Token::Ampersand => todo!("binary double amp and"),
            Token::Or => todo!("binary or"),
            Token::Pipe => todo!("binary double pipe or"),
            Token::Xor => todo!("binary xor"),
            Token::Caret => todo!("binary double caret xor"),
            Token::Dot => todo!("binary member access"),

            Token::LParen => self.parse_tuple()?,
            _ => bail!("syntax error: parse_variable_declaration")
        })
    }

    fn parse_tuple(&mut self) -> Result<Expression> { todo!("parse_tuple") }
    fn parse_identifier_statement(&mut self, _ident: &String) -> Result<Expression> { todo!("parse_identifier_statement") }
    fn parse_function_declaration(&mut self) -> Result<Declaration> { todo!("parse_function_declaration") }
    fn parse_struct_declaration(&mut self) -> Result<Declaration> { todo!("parse_struct_declaration") }
    fn parse_enum_declaration(&mut self) -> Result<Declaration> { todo!("parse_enum_declaration") }
    fn parse_object_declaration(&mut self) -> Result<Declaration> { todo!("parse_object_declaration") }
    fn parse_if_statement(&mut self) -> Result<ASTNode> { todo!("parse_if_statement") }
    fn parse_elif_statement(&mut self) -> Result<ASTNode> { todo!("parse_elif_statement") }
    fn parse_match_statement(&mut self) -> Result<ASTNode> { todo!("parse_match_statement") }
    fn parse_import_statement(&mut self) -> Result<ASTNode> { todo!("parse_import_statement") }
    fn parse_parameters(&mut self) -> Result<Vec<Expression>> { todo!("parse_parameters") }
    fn parse_arguments(&mut self) -> Result<Vec<Expression>> { todo!("parse_arguments") }
    fn parse_anonymouse_function(&mut self) -> Result<Expression> { todo!("parse_anonymous_function") }
}

