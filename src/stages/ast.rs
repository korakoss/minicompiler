use crate::shared::newtyping::*;
use crate::shared::binops::*;

use std::collections::HashMap;


#[derive(Debug, Clone)]
pub struct ASTProgram {
    pub typetable: TypeTable,
    pub functions: HashMap<FuncSignature, ASTFunction>,
}


#[derive(Debug, Clone)]
pub struct ASTFunction {
    pub name: String,
    pub args: HashMap<String, Type>,
    pub body: Vec<ASTStatement>,
    pub ret_type: Type,
}

impl ASTFunction {
   
    pub fn get_signature(&self) -> FuncSignature {
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
        var: Variable,
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
   FieldAccess {
       of: Box<ASTLValue>,
       field: String,
    },
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
        typ: Type,
        fields: HashMap<String, ASTExpression>,
    },
}



