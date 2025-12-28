use crate::common::*;
use std::{collections::HashMap};


#[derive(Clone, Debug)]
pub struct HIRProgram {
    pub functions: HashMap<FuncId, HIRFunction>,
    pub entry: Option<FuncId>,
}


#[derive(Clone, Debug)]
pub struct HIRFunction {
    pub args: Vec<VarId>,
    pub body: Vec<HIRStatement>,
    pub variables: HashMap<VarId, Variable>,
    pub ret_type: Type,
}


#[derive(Clone, Debug)]
pub enum HIRStatement {
    Let {
        var: Place,     
        value: HIRExpression,
    },
    Assign {
        target: Place,  
        value: HIRExpression,
    },
    If {
        condition: HIRExpression, 
        if_body: Vec<HIRStatement>,    
        else_body: Option<Vec<HIRStatement>>,
    },
    While {
        condition: HIRExpression,
        body: Vec<HIRStatement>,
},
    Break,
    Continue,
    Return(HIRExpression),
    Print(HIRExpression),
}


#[derive(Clone, Debug)]
pub struct HIRExpression {
    pub typ: Type,
    pub expr: HIRExpressionKind,
}

#[derive(Clone, Debug)]
pub enum HIRExpressionKind {
    IntLiteral(i32),
    Variable(VarId),
    BinOp {
        op: BinaryOperator,
        left: Box<HIRExpression>,
        right: Box<HIRExpression>,
    },
    FuncCall {
        funcid: FuncId,
        args: Vec<HIRExpression>,
    },
    BoolTrue,
    BoolFalse,
    Struct {
        fields: HashMap<String, HIRExpression>
    }
}


#[derive(Clone, Debug)]
pub enum Place {
    Variable(VarId),
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct VarId(pub usize);

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct FuncId(pub usize);

