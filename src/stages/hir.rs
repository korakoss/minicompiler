use crate::shared::newtyping::*;
use crate::shared::binops::*;
use crate::stages::common::FuncId;

use std::{collections::HashMap};

#[derive(Clone, Debug)]
pub struct HIRProgram {
    pub typetable: TypeTable, 
    pub functions: HashMap<FuncId, HIRFunction>,
    pub entry: FuncId,
}


#[derive(Clone, Debug)]
pub struct HIRFunction {
    pub name: String,
    pub args: Vec<VarId>,
    pub variables: HashMap<VarId, Variable>,
    pub body: Vec<HIRStatement>,
    pub ret_type: Type,
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
    pub typ: Type,
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
}

#[derive(Clone, Debug)]
pub struct Place {
    pub typ: Type,
    pub place: PlaceKind,
}

#[derive(Clone, Debug)]
pub enum PlaceKind {
    Variable(VarId),
    StructField {
        of: Box<Place>,
        field: String,
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct VarId(pub usize);


