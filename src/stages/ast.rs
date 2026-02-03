use std::collections::HashMap;

use crate::shared::{
    binops::BinaryOperator, tables::GenericTypetable, typing::{GenericType, TypevarId}, utils::*
};


#[derive(Debug, Clone)]
pub struct ASTProgram {
    pub typetable: GenericTypetable,
    pub functions: HashMap<GenericFuncSignature, ASTFunction>,
}


#[derive(Debug, Clone)]
pub struct ASTFunction {
    pub name: String,
    pub typvars: Vec<TypevarId>,
    pub args: HashMap<String, GenericType>,    // TODO: Does this lose argument order?
    pub body: Vec<ASTStatement>,
    pub ret_type: GenericType,
}

impl ASTFunction {
   
    pub fn get_signature(&self) -> GenericFuncSignature {
        FuncSignature { 
            name: self.name.clone(), 
            typevars: self.typvars.clone(),
            argtypes: self.args.values().cloned().collect()
        }
    }
}



#[derive(Debug, Clone)]
pub enum ASTStatement {
    Let {
        var: GenTypeVariable,
        value: ASTExpression,
    },
    Assign {
        target: ASTLValue,
        value: ASTExpression
    },
    If {
        condition: ASTExpression,
        if_body: Vec<ASTStatement>,
        else_body: Option<Vec<ASTStatement>>,
    },
    While {
        condition: ASTExpression,
        body: Vec<ASTStatement>,
    },
    Break,
    Continue,
    Return(ASTExpression),
    Print(ASTExpression),
}

#[derive(Debug, Clone)]
pub enum ASTLValue {        // Rename to ASTPlace
   Variable(String),
   FieldAccess {            // MAybe change to a chain
       of: Box<ASTLValue>,
       field: String,
    },
    Deref(ASTExpression)
}

#[derive(Debug, Clone)]
pub enum ASTExpression {
    IntLiteral(i32),
    Variable(String),
    BinOp {
       op: BinaryOperator,
       left: Box<ASTExpression>,
       right: Box<ASTExpression>,
    },
    FuncCall {
        funcname: String,
        type_params: Vec<GenericType>,
        args: Vec<ASTExpression>,
    },
    BoolTrue,
    BoolFalse,
    
    FieldAccess {
        expr: Box<ASTExpression>,
        field: String,
    },

    StructLiteral {
        typ: GenericType,
        fields: HashMap<String, ASTExpression>,
    },

    Reference(Box<ASTExpression>),
    Dereference(Box<ASTExpression>)
}


