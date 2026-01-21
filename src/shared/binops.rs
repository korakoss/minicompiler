use crate::shared::typing::*;

#[derive(Debug, Clone)]
pub enum BinaryOperator {
    Add, 
    Sub, 
    Mul, 
    Equals,
    Less,       
    Modulo
}

pub fn binop_typecheck(op: &BinaryOperator, left_type: &ConcreteType, right_type: &ConcreteType) -> Option<ConcreteType> {
    
    match op {
        &BinaryOperator::Add | &BinaryOperator::Sub | &BinaryOperator::Mul| &BinaryOperator::Modulo=>{
            if left_type == &ConcreteType::Prim(PrimType::Integer) && right_type == &ConcreteType::Prim(PrimType::Integer){
                Some(ConcreteType::Prim(PrimType::Integer))
            } else {
                None
            }
        }
        &BinaryOperator::Equals => {
            if left_type == right_type {
                // TODO: careful later
                Some(ConcreteType::Prim(PrimType::Bool))
            } else {
                None
            }
        }
        &BinaryOperator::Less => {
            if left_type == &ConcreteType::Prim(PrimType::Integer) && right_type == &ConcreteType::Prim(PrimType::Integer){
                Some(ConcreteType::Prim(PrimType::Bool))
            } else {
                None
            }
        } 
    }
}


