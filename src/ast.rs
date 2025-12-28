use crate::common::*;
use std::collections::HashMap;


type UASTProgram = ASTProgram<DeferredType>;
type UASTFunction = ASTFunction<DeferredType>;
type UASTStatement = ASTStatement<DeferredType>;

type TASTProgram = ASTProgram<Type>;
type TASTFunction = ASTFunction<Type>;
type TASTStatement = ASTStatement<Type>;

type DeferredTypeVariable = Variable<DeferredType>;
type TypedVariable = Variable<Type>;


#[derive(Debug, Clone)]
pub struct ASTProgram<T> {
    pub struct_defs: HashMap<TypeIdentifier, NewType<T>>,
    pub functions: Vec<ASTFunction<T>>,
}

#[derive(Debug, Clone)]
pub struct ASTFunction<T> {
    pub name: String,
    pub args: Vec<Variable<T>>,
    pub body: Vec<ASTStatement<T>>,
    pub ret_type: T,
}

#[derive(Debug, Clone)]
pub enum ASTStatement<T> {
    Let {
        var: Variable<T>,
        value: ASTExpression,
    },
    LetStruct {
        var: Variable<T>,
        value: ASTStructLiteral<T>,
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
        args: Vec<Box<ASTExpression>>,
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


/*
impl TASTFunction {
   
    pub fn get_signature(&self) -> FuncSignature {
        FuncSignature { 
            name: self.name.clone(), 
            argtypes: self.args.iter().map(|x| x.typ.clone()).collect(), 
} 
    }
}
*/
