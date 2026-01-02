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

pub fn binop_typecheck(op: &BinaryOperator, left_type: &Type, right_type: &Type) -> Option<Type> {
    
    match op {
        &BinaryOperator::Add | &BinaryOperator::Sub | &BinaryOperator::Mul| &BinaryOperator::Modulo=>{
            if left_type == &Type::Prim(PrimitiveType::Integer) && right_type == &Type::Prim(PrimitiveType::Integer){
                Some(Type::Prim(PrimitiveType::Integer))
            } else {
                None
            }
        }
        &BinaryOperator::Equals => {
            if left_type == right_type {
                // TODO: careful later
                Some(Type::Prim(PrimitiveType::Bool))
            } else {
                None
            }
        }
        &BinaryOperator::Less => {
            if left_type == &Type::Prim(PrimitiveType::Integer) && right_type == &Type::Prim(PrimitiveType::Integer){
                Some(Type::Prim(PrimitiveType::Bool))
            } else {
                None
            }
        } 
    }
}


