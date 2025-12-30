use crate::shared::typing::*;
use crate::shared::binops::*;

use std::collections::HashMap;


pub type UASTProgram = ASTProgram<DeferredType>;
pub type UASTFunction = ASTFunction<DeferredType>;
pub type UASTStatement = ASTStatement<DeferredType>;

pub type TASTProgram = ASTProgram<Type>;
pub type TASTFunction = ASTFunction<Type>;
pub type TASTStatement = ASTStatement<Type>;


#[derive(Debug, Clone)]
pub struct ASTProgram<T> {
    pub new_types: HashMap<TypeIdentifier, TypeConstructor<T>>,
    pub functions: HashMap<FuncSignature<T>, ASTFunction<T>>,
}

#[derive(Debug, Clone)]
pub struct ASTFunction<T> {
    pub name: String,
    pub args: HashMap<String, T>,
    pub body: Vec<ASTStatement<T>>,
    pub ret_type: T,
}

impl<T: Clone> ASTFunction<T> {
   
    pub fn get_signature(&self) -> FuncSignature<T> {
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
pub enum ASTStatement<T> {
    Let {
        var: Variable<T>,
        value: ASTExpression,
    },
    Assign {
        target: ASTExpression,         
        value: ASTExpression
    },
    If {
        condition: ASTExpression,
        if_body: Vec<ASTStatement<T>>,
        else_body: Option<Vec<ASTStatement<T>>>,
    },
    While {
        condition: ASTExpression,
        body: Vec<ASTStatement<T>>,
    },
    Break,
    Continue,
    Return(ASTExpression),
    Print(ASTExpression),
}

#[derive(Debug, Clone)]
pub struct ASTStructLiteral<T> {
    pub typ: T,
    pub fields: HashMap<String, ASTExpression>,
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
        fields: HashMap<String, ASTExpression>,
    },
}



