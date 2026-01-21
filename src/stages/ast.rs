use crate::shared::typing::*;
use crate::shared::binops::*;
use crate::shared::tables::*;
use crate::shared::utils::*;

use std::collections::HashMap;


#[derive(Debug, Clone)]
pub struct ASTProgram {
    pub typetable: GenericTypetable,
    pub functions: HashMap<ConcreteFuncSignature, ASTFunction>,
}


#[derive(Debug, Clone)]
pub struct ASTFunction {
    pub name: String,
    pub args: HashMap<String, ConcreteType>,    // Does this lose argument order?
    pub body: Vec<ASTStatement>,
    pub ret_type: ConcreteType,
}

impl ASTFunction {
   
    pub fn get_signature(&self) -> ConcreteFuncSignature {
        FuncSignature { 
            name: self.name.clone(), 
            argtypes: self.args
                .iter()
                .map(|(_, argt)| argt.clone())
                .collect()
        }
    }
}



#[derive(Debug, Clone)]
pub enum ASTStatement {
    Let {
        var: ConcreteVariable,
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
        args: Vec<ASTExpression>,
    },
    BoolTrue,
    BoolFalse,
    
    FieldAccess {
        expr: Box<ASTExpression>,
        field: String,
    },

    StructLiteral {
        typ: ConcreteType,
        fields: HashMap<String, ASTExpression>,
    },

    Reference(Box<ASTExpression>),
    Dereference(Box<ASTExpression>)
}



