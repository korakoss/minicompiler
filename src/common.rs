use std::{collections::HashMap};

#[derive(PartialEq,Eq, Hash, Debug, Clone)]
pub enum Type {
    Integer,
    Bool,
    None,
    Struct {
        name: String,
        fields: Vec<(String, Type)>,
    }
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub name: String,
    pub typ: Type,
    // TODO: mutable, etc
}

#[derive(Debug, Clone)]
pub enum BinaryOperator {
    Add, 
    Sub, 
    Mul, 
    Equals,
    Less,       // NOTE: represents left < right 
    Modulo

    // TODO
        //Greater, 
        //Div (later, when floats ig?),
        //NotEqual
}

// TODO: eventually also UnaryOperation (eg. negation)


pub fn binop_typecheck(op: &BinaryOperator, left_type: &Type, right_type: &Type) -> Option<Type> {
    
    match op {
        &BinaryOperator::Add | &BinaryOperator::Sub | &BinaryOperator::Mul| &BinaryOperator::Modulo=>{
            if left_type == &Type::Integer && right_type == &Type::Integer {
                Some(Type::Integer)
            } else {
                None
            }
        }
        &BinaryOperator::Equals => {
            if left_type == right_type {
                // TODO: careful later
                Some(Type::Bool)
            } else {
                None
            }
        }
        &BinaryOperator::Less => {
            if left_type == &Type::Integer && right_type == &Type::Integer {
                Some(Type::Bool)
            } else {
                None
            }
        } 
    }
}


#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FuncSignature {
    pub name: String,
    pub argtypes: Vec<Type>,
    // NOTE: maybe return type sometime?
}
