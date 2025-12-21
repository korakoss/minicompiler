use crate::common::*;
use std::{collections::HashMap};
use crate::ast::*;


// AFTER TYPING+VARSCOPING


#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct VarId(pub usize);

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct ScopeId(pub usize);

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct FuncId(pub usize);

#[derive(Clone)]
pub enum TypedExpressionKind {
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
    }
}

#[derive(Clone)]
pub struct HIRExpression {
    pub typ: Type,
    pub expr: TypedExpressionKind,
}

#[derive(Clone)]
pub enum Place {
    Variable(VarId),
}


#[derive(Clone)]
pub struct ScopeBlock {      
    pub parent_id: Option<ScopeId>,
    pub scope_vars: HashMap<String, VarId>,
    pub within_func: bool,
    pub within_loop: bool,
    pub statements: Vec<HIRStatement>,
}

#[derive(Clone)]
pub enum HIRStatement {
    Let {
        var: Place,     // Expected to be Variable
        value: HIRExpression,
    },
    Assign {
        target: HIRExpression,   // Expected to be l-value
        value: HIRExpression,
    },
    If {
        condition: HIRExpression,    // Expected to be Boolean
        if_body: ScopeBlock,    
        else_body: Option<ScopeBlock>,
    },
    While {
        condition: HIRExpression,
        body: ScopeBlock,
},
    Break,
    Continue,
    Return(HIRExpression),
    Print(HIRExpression),
}


pub struct HIRFunction {
    pub args: Vec<VariableInfo>,
    pub body: ScopeBlock,
    pub ret_type: Type,
}



