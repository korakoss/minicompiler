use crate::shared::typing::*;
use std::{collections::HashMap};
use crate::shared::binops::*;


#[derive(Clone, Debug)]
pub struct HIRProgram {
    pub new_types: HashMap<TypeIdentifier,DerivType>,
    pub functions: HashMap<FuncId, HIRFunction>,
    pub entry: FuncId,
}


#[derive(Clone, Debug)]
pub struct HIRFunction {
    pub name: String,
    pub args: Vec<VarId>,
    pub variables: HashMap<VarId, TypedVariable>,
    pub body: Vec<HIRStatement>,
    pub ret_type: Type,
}


#[derive(Clone, Debug)]
pub enum HIRStatement {
    Let {
        var: Place,                 // TODO: Maybe change to VarId for clarity 
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


#[derive(Debug, Clone)]
pub enum HIRExpression {
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
        typ: Type,
        fields: HashMap<String, HIRExpression>,
    },
}

#[derive(Clone, Debug)]
pub enum Place {
    Variable(VarId),
    StructField {
        of: HIRExpression,
        field: String,
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct VarId(pub usize);

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct FuncId(pub usize);
