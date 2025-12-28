use std::collections::HashMap;
use crate::common::*;


#[derive(Debug, Clone)]
pub struct UASTProgram {
    pub struct_defs: HashMap<TypeIdentifier, UASTStructDef>,
    pub functions: Vec<UASTFunction>,
}

#[derive(Debug, Clone)]
pub struct UASTFunction {
    pub name: String,
    pub args: Vec<UASTVariable>,
    pub body: Vec<UASTStatement>,
    pub ret_type: TypeIdentifier,
}

#[derive(Debug, Clone)]
pub enum UASTStatement {
    Let {
        var: UASTVariable,
        value: UASTExpression,
        // TODO: type field
    },
    LetStruct {
        var: UASTVariable,
        value: UASTStruct,
    },
    Assign {
        target: UASTExpression,         
        value: UASTExpression
    },
    If {
        condition: UASTExpression,
        if_body: Vec<UASTStatement>,
        else_body: Option<Vec<UASTStatement>>,
    },
    While {
        condition: UASTExpression,
        body: Vec<UASTStatement>,
    },
    Break,
    Continue,
    Return(UASTExpression),
    Print(UASTExpression),
}

#[derive(Debug, Clone)]
pub enum UASTExpression {
    IntLiteral(i32),
    Variable(String),
    BinOp {
       op: BinaryOperator,
       left: Box<UASTExpression>,
       right: Box<UASTExpression>,
    },
    FuncCall {
        funcname: String,
        args: Vec<Box<UASTExpression>>,
    },
    BoolTrue,
    BoolFalse,

    FieldAccess {
        expr: Box<UASTExpression>,
        field: String,
    }
    // TODO: negation 
}


#[derive(Debug, Clone)]
pub struct UASTStructDef {
    pub fields: HashMap<String, TypeIdentifier>
}

#[derive(Debug, Clone)]
pub struct UASTStruct {
    pub retar_type: TypeIdentifier,
    pub fields: HashMap<String, UASTExpression>,
}

#[derive(Debug, Clone)]
pub struct UASTVariable {
    pub name: String,
    pub retar_type: TypeIdentifier,
}

