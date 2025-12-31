use crate::shared::typing::*;
use std::{collections::HashMap};
use crate::ast::*;


#[derive(Clone, Debug)]
pub struct HIRProgram {
    pub functions: HashMap<CompleteFunctionSignature, HIRFunction>,
    pub entry: CompleteFunctionSignature,
    // pub layouts: HashMap<Type, LayoutInfo>,
    pub variables: HashMap<VarId, TypedVariable>,
}


#[derive(Clone, Debug)]
pub struct HIRFunction {
    pub name: String,
    pub args: Vec<VarId>,
    pub body: Vec<HIRStatement>,
    pub ret_type: Type,
}


#[derive(Clone, Debug)]
pub enum HIRStatement {
    Let {
        var: Place,     
        value: TASTExpression,
    },
    Assign {
        target: Place,  
        value: TASTExpression,
    },
    If {
        condition: TASTExpression, 
        if_body: Vec<HIRStatement>,    
        else_body: Option<Vec<HIRStatement>>,
    },
    While {
        condition: TASTExpression, 
        body: Vec<HIRStatement>,
},
    Break,
    Continue,
    Return(TASTExpression),
    Print(TASTExpression),
}


#[derive(Clone, Debug)]
pub enum Place {
    Variable(VarId),
    // TODO: StructField
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct VarId(pub usize);

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct FuncId(pub usize);

#[derive(Clone, Debug)]
pub struct HIRVariable {
    name: String,           // Not directly necessary, but for errors later
    typ: Type,
}

#[derive(Clone, Debug)]
pub enum LayoutInfo {
    Primitive(usize),               // Variable size
    Struct {
        field_offsets: HashMap<String, usize>
    }
}

