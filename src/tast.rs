use std::collections::HashMap;

use crate::common::*;

#[derive(Debug, Clone)]
pub struct TASTProgram {
    pub struct_defs: HashMap<TypeIdentifier, NewType>,
    pub functions: Vec<TASTFunction>,
}

#[derive(Debug, Clone)]
pub struct TASTFunction {
    pub name: String,
    pub args: Vec<Variable>,
    pub body: Vec<TASTStatement>,
    pub ret_type: Type,
}

impl TASTFunction {
   
    pub fn get_signature(&self) -> FuncSignature {
        FuncSignature { 
            name: self.name.clone(), 
            argtypes: self.args.iter().map(|x| x.typ.clone()).collect(), 
        } 
    }
}

#[derive(Debug, Clone)]
pub enum TASTStatement {
    Let {
        var: Variable,
        value: TASTExpression,
        // TODO: type field
    },
    LetStruct {
        var: Variable,
        value: TASTStruct,
    },
    Assign {
        target: TASTExpression,         
        value: TASTExpression
    },
    If {
        condition: TASTExpression,
        if_body: Vec<TASTStatement>,
        else_body: Option<Vec<TASTStatement>>,
    },
    While {
        condition: TASTExpression,
        body: Vec<TASTStatement>,
    },
    Break,
    Continue,
    Return(TASTExpression),
    Print(TASTExpression),
}

#[derive(Debug, Clone)]
pub enum TASTExpression {
    IntLiteral(i32),
    Variable(String),
    BinOp {
       op: BinaryOperator,
       left: Box<TASTExpression>,
       right: Box<TASTExpression>,
    },
    FuncCall {
        funcname: String,
        args: Vec<Box<TASTExpression>>,
    },
    BoolTrue,
    BoolFalse,
    
    FieldAccess {
        expr: Box<TASTExpression>,
        field: String,
    },

    StructLiteral {
        fields: HashMap<String, TASTExpression>,
    },
    // TODO: negation 
}

#[derive(Debug, Clone)]
pub struct TASTStruct {
    pub typ: Type,
    pub fields: HashMap<String, TASTExpression>,
}


