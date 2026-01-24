use crate::shared::binops::*;


#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Operators
    Assign,
    Plus,
    Minus,
    Multiply,
    Equals,
    Less,
    Modulo,

    // Delimiters
    Semicolon,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftSqBracket,
    RightSqBracket,
    Comma,
    Colon,
    Dot,

    // Values 
    Identifier(String),

    // Literals
    // TODO: None
    True,
    False,
    IntLiteral(i32),

    // Keywords
    Print,
    If,
    Else,
    While,
    Break,
    Continue,
    Function,
    Return,
    Let,
    Struct,
    
    // Type stuff
    Int, 
    Bool,
    RightArrow,

    Ref,
    Deref,
}




pub fn get_connector_precedence(op_token: &Token) -> usize {
    match *op_token {
        Token::Dot => 3, 
        Token::Multiply | Token::Modulo => 2,
        Token::Plus| Token::Minus => 1,
        Token::Equals | Token::Less => 0,
        _ => panic!("Expected binary operator token, found: {:?}", op_token), 
    }
}

pub fn map_binop_token(op_token: &Token) -> BinaryOperator {
    match *op_token {
        Token::Plus => BinaryOperator::Add,
        Token::Minus => BinaryOperator::Sub,
        Token::Multiply => BinaryOperator::Mul,
        Token::Equals => BinaryOperator::Equals,
        Token::Less => BinaryOperator::Less,
        Token::Modulo => BinaryOperator::Modulo,
        _ => panic!("Expected binary operator token, found: {:?}", op_token), 

    }
}

