use std::collections::BTreeMap;

pub struct SizedVariable {
    var: TypedVariable,
    size: usize,
}

pub type DeferredTypeVariable = Variable<DeferredType>;
pub type TypedVariable = Variable<Type>;

pub type DeferredNewType = NewType<DeferredType>;
pub type CompleteNewType = NewType<Type>;

#[derive(Debug, Clone)]
pub struct Variable<T> {
    pub name: String,
    pub typ: T,
    // TODO: mutable, etc
}

#[derive(PartialEq,Eq, Debug, Hash, Clone)]
pub enum Type {
    Integer,
    Bool,
    None,
    NewType(NewType<Type>),
}

#[derive(Debug, Clone)]
pub enum DeferredType {
    Resolved(Type),
    Unresolved(TypeIdentifier)
}

#[derive(PartialEq,Eq, Debug, Hash, Clone)]
pub enum NewType<T> {
    Struct {
        fields: BTreeMap<String, T>,
    }
}


#[derive(Debug, Clone)]
pub enum BinaryOperator {
    Add, 
    Sub, 
    Mul, 
    Equals,
    Less,       // NOTE: represents left < right 
    Modulo

    // TODO
        //Greater, 
        //Div (later, when floats ig?),
        //NotEqual
}

// TODO: eventually also UnaryOperation (eg. negation)



#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FuncSignature {
    pub name: String,
    pub argtypes: Vec<Type>,
    // NOTE: maybe return type sometime?
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct TypeIdentifier(pub String); 





pub fn binop_typecheck(op: &BinaryOperator, left_type: &Type, right_type: &Type) -> Option<Type> {
    
    match op {
        &BinaryOperator::Add | &BinaryOperator::Sub | &BinaryOperator::Mul| &BinaryOperator::Modulo=>{
            if left_type == &Type::Integer && right_type == &Type::Integer {
                Some(Type::Integer)
            } else {
                None
            }
        }
        &BinaryOperator::Equals => {
            if left_type == right_type {
                // TODO: careful later
                Some(Type::Bool)
            } else {
                None
            }
        }
        &BinaryOperator::Less => {
            if left_type == &Type::Integer && right_type == &Type::Integer {
                Some(Type::Bool)
            } else {
                None
            }
        } 
    }
}


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
}



// TODO: instead of these two funcs, maybe we should make a big binop info table with rows like: Token, Binoptype,precedence 

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

