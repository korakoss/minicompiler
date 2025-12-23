use crate::common::*;


#[derive(Debug, Clone)]
pub enum ASTExpression {
    IntLiteral(i32),
    Variable(String),
    BinOp {
       op: BinaryOperator,
       left: Box<ASTExpression>,
       right: Box<ASTExpression>,
    },
    FuncCall {
        funcname: String,
        args: Vec<Box<ASTExpression>>,
    },
    BoolTrue,
    BoolFalse,
    // TODO: negation 
}


#[derive(Debug, Clone)]
pub enum ASTStatement {
    Let {
        var: Variable,
        value: ASTExpression,
        // TODO: type field
    },
    Assign {
        target: ASTExpression,         
        value: ASTExpression
    },
    If {
        condition: ASTExpression,
        if_body: Vec<ASTStatement>,
        else_body: Option<Vec<ASTStatement>>,
    },
    While {
        condition: ASTExpression,
        body: Vec<ASTStatement>,
    },
    Break,
    Continue,
    Return(ASTExpression),
    Print(ASTExpression),
}

#[derive(Debug, Clone)]
pub struct ASTFunction {
    pub name: String,
    pub args: Vec<Variable>,
    pub body: Vec<ASTStatement>,
    pub ret_type: Type,
}

#[derive(Debug, Clone)]
pub struct ASTProgram {
    pub functions: Vec<ASTFunction>,
    pub main_statements: Vec<ASTStatement>
}


