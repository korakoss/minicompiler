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

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
pub struct HIRExpression {
    pub typ: Type,
    pub expr: TypedExpressionKind,
}

#[derive(Clone, Debug)]
pub enum Place {
    Variable(VarId),
}


#[derive(Clone, Debug)]
pub struct ScopeBlock {      
    pub parent_id: Option<ScopeId>,
    pub scope_vars: HashMap<String, VarId>,
    pub within_func: bool,
    pub within_loop: bool,
    pub statements: Vec<HIRStatement>,
}

#[derive(Clone, Debug)]
pub enum HIRStatement {
    Let {
        var: Place,     // Expected to be Variable
        value: HIRExpression,
    },
    Assign {
        target: Place,   // Expected to be l-value
        value: HIRExpression,
    },
    If {
        condition: HIRExpression,    // Expected to be Boolean
        if_body: ScopeId,    
        else_body: Option<ScopeId>,
    },
    While {
        condition: HIRExpression,
        body: ScopeId,
},
    Break,
    Continue,
    Return(HIRExpression),
    Print(HIRExpression),
}

#[derive(Clone, Debug)]
pub struct HIRFunction {
    pub args: Vec<VariableInfo>,
    pub body: ScopeId,
    pub ret_type: Type,
}

#[derive(Clone, Debug)]
pub struct HIRProgram {
    pub scopes: HashMap<ScopeId, ScopeBlock>,
    pub variables: HashMap<VarId, VariableInfo>,
    pub functions: HashMap<FuncId, HIRFunction>,
    pub global_scope: ScopeId,
}


