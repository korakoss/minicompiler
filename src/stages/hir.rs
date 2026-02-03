use std::{collections::HashMap};

use crate::shared::{
    binops::BinaryOperator, 
    tables::GenericTypetable, 
    typing::{GenericType, TypevarId}, 
    utils::{GenTypeVariable},
    ids::FuncId,
    callgraph::CallGraph,
};


#[derive(Clone, Debug)]
pub struct HIRProgram {
    pub typetable: GenericTypetable, 
    pub call_graph: CallGraph,
    pub functions: HashMap<FuncId, HIRFunction>,
    pub entry: FuncId,
}


#[derive(Clone, Debug)]
pub struct HIRFunction {
    pub name: String,
    pub typvars: Vec<TypevarId>,
    pub args: Vec<VarId>,
    pub variables: HashMap<VarId, GenTypeVariable>,
    pub body: Vec<HIRStatement>,
    pub ret_type: GenericType,
}


#[derive(Clone, Debug)]
pub enum HIRStatement {
    Let {
        var: VarId,
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
    Return(Option<HIRExpression>),
    Print(HIRExpression),
}


#[derive(Debug, Clone)]
pub struct HIRExpression {
    pub typ: GenericType,
    pub expr: HIRExpressionKind,
}

#[derive(Debug, Clone)]
pub enum HIRExpressionKind {
    IntLiteral(i32),
    Variable(VarId),
    BinOp {
       op: BinaryOperator,
       left: Box<HIRExpression>,
       right: Box<HIRExpression>,
    },
    FuncCall {
        id: FuncId, 
        type_params: Vec<GenericType>,
        args: Vec<HIRExpression>,
    },
    BoolTrue,
    BoolFalse,
    FieldAccess {
        expr: Box<HIRExpression>,
        field: String,
    },
    StructLiteral {
        fields: HashMap<String, HIRExpression>,
    },
    Reference(Box<HIRExpression>),
    Dereference(Box<HIRExpression>),
}

#[derive(Clone, Debug)]
pub struct Place {
    pub typ: GenericType,
    pub place: PlaceKind,
}

#[derive(Clone, Debug)]
pub enum PlaceKind {
    Variable(VarId),
    StructField {
        of: Box<Place>,
        field: String,
    },
    Deref(HIRExpression),
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct VarId(pub usize);
