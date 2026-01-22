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

pub fn binop_typecheck(op: &BinaryOperator, left_type: &GenericType, right_type: &GenericType) -> Option<GenericType> {
    
    match op {
        &BinaryOperator::Add | &BinaryOperator::Sub | &BinaryOperator::Mul| &BinaryOperator::Modulo=>{
            if left_type == &GenericType::Prim(PrimType::Integer) && right_type == &GenericType::Prim(PrimType::Integer){
                Some(GenericType::Prim(PrimType::Integer))
            } else {
                None
            }
        }
        &BinaryOperator::Equals => {
            if left_type == right_type {
                // TODO: careful later
                Some(GenericType::Prim(PrimType::Bool))
            } else {
                None
            }
        }
        &BinaryOperator::Less => {
            if left_type == &GenericType::Prim(PrimType::Integer) && right_type == &GenericType::Prim(PrimType::Integer){
                Some(GenericType::Prim(PrimType::Bool))
            } else {
                None
            }
        } 
    }
}


