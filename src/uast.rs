use std::collections::HashMap;
use crate::common::*;


#[derive(Debug, Clone)]
pub struct UASTProgram {
    pub type_defs: HashMap<TypeIdentifier, DeferredNewType>,
    pub functions: Vec<UASTFunction>,
}

#[derive(Debug, Clone)]
pub struct UASTFunction {
    pub name: String,
    pub args: Vec<DeferredTypeVariable>,
    pub body: Vec<UASTStatement>,
    pub ret_type: DeferredType,
}

#[derive(Debug, Clone)]
pub enum UASTStatement {
    Let {
        var: DeferredTypeVariable,
        value: UASTExpression,
        // TODO: type field
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
    },

    StructLiteral {
        fields: HashMap<String, UASTExpression>,
    }
    // TODO: negation 
}

#[derive(Debug, Clone)]
pub enum DeferredType {
    Resolved(Type),
    Unresolved(TypeIdentifier)
}

#[derive(Debug, Clone)]
pub enum DeferredNewType {
    Struct {
        fields: HashMap<String, DeferredType>
    }
}

#[derive(Debug, Clone)]
pub struct UASTStructLiteral {
    pub retar_type: DeferredType,
    pub fields: HashMap<String, UASTExpression>,
}

#[derive(Debug, Clone)]
pub struct DeferredTypeVariable {
    pub name: String,
    pub retar_type: DeferredType,
}

