use crate::common::*;

#[derive(Debug, Clone)]
pub enum BinaryOperator {
    Add, 
    Sub, 
    Mul, 
    Equals,
    Less,       // left < right 
    Modulo
    //Greater, 
    //Div (later, when floats ig?),
    //NotEqual
}


#[derive(Debug, Clone)]
pub enum Expression {
    IntLiteral(i32),

    Variable(String),

    BinOp {
       op: BinaryOperator,
       left: Box<Expression>,
       right: Box<Expression>,
    },

    FuncCall {
        funcname: String,
        args: Vec<Box<Expression>>,
    }
    
    // UnaryOp (eg. negation)
}


#[derive(Debug, Clone)]
pub enum Statement {

    Let {
        var: Variable,
        value: Expression,
        // TODO: type field
    },
    Assign {
        target: Expression,         // validate lvalue
        value: Expression
    },
    If {
        condition: Expression,
        if_body: Vec<Statement>,
        else_body: Option<Vec<Statement>>,
    },
    While {
        condition: Expression,
        body: Vec<Statement>,
    },
    Break,
    Continue,
    Return(Expression),
    Print(Expression),
}


#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub args: Vec<Variable>,
    pub body: Vec<Statement>,
    pub ret_type: Type,
}


#[derive(Debug, Clone)]
pub struct RawAST {
    pub functions: Vec<Function>,
    pub main_statements: Vec<Statement>
}


