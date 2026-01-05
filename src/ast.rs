use crate::shared::typing::*;
use crate::shared::binops::*;

use std::collections::HashMap;


#[derive(Debug, Clone)]
pub struct TASTProgram {
    pub typetable: TypeTable,
    pub functions: HashMap<CompleteFunctionSignature, TASTFunction>,
}

pub type TASTFunction = ASTFunction<Type>;
pub type TASTStatement = ASTStatement<Type>;
pub type TASTExpression = ASTExpression<Type>;


#[derive(Debug, Clone)]
pub struct UASTProgram {
    pub new_types: HashMap<TypeIdentifier, DeferredDerivType>,
    pub functions: HashMap<DeferredFunctionSignature, UASTFunction>,
}

pub type UASTFunction = ASTFunction<DeferredType>;
pub type UASTStatement = ASTStatement<DeferredType>;
pub type UASTExpression = ASTExpression<DeferredType>;


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
        value: ASTExpression<T>,
    },
    Assign {
        target: ASTLValue,
        value: ASTExpression<T>
    },
    If {
        condition: ASTExpression<T>,
        if_body: Vec<ASTStatement<T>>,
        else_body: Option<Vec<ASTStatement<T>>>,
    },
    While {
        condition: ASTExpression<T>,
        body: Vec<ASTStatement<T>>,
    },
    Break,
    Continue,
    Return(ASTExpression<T>),
    Print(ASTExpression<T>),
}

#[derive(Debug, Clone)]
pub enum ASTLValue {
   Variable(String),
   FieldAccess {
       of: Box<ASTLValue>,
       field: String,
    },
}

#[derive(Debug, Clone)]
pub enum ASTExpression<T> {
    IntLiteral(i32),
    Variable(String),
    BinOp {
       op: BinaryOperator,
       left: Box<ASTExpression<T>>,
       right: Box<ASTExpression<T>>,
    },
    FuncCall {
        funcname: String,
        args: Vec<ASTExpression<T>>,
    },
    BoolTrue,
    BoolFalse,
    
    FieldAccess {
        expr: Box<ASTExpression<T>>,
        field: String,
    },

    StructLiteral {
        typ: T,
        fields: HashMap<String, ASTExpression<T>>,
    },
}



