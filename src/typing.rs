use crate::ast::*;
use crate::analyzing::*;


#[derive(Clone)]
pub enum Type {
    Int,
    Bool,
    None,
    // Pointer,
    // Struct,
    // Array
}



fn type_binop(binop: BinaryOperator, left_type: Type, right_type: Type) -> Result<Type, String> {
   match binop {
       BinaryOperator::Add |  BinaryOperator::Sub | BinaryOperator::Mul | BinaryOperator::Modulo => {
            if left_type == Type::Int && right_type == Type::Int {
                return Ok(Type::Int);
            } else {
                return Err("Attempted numeric binop with not both operands having Int type"); 
            }
       },
       BinaryOperator::Less => {
            if left_type == Type::Int && right_type == Type::Int {
                return Ok(Type::Bool);
            } else {
                return Err("Attempted numeric comparison with not both operands having Int type"); 
            }
        },
        BinaryOperator::Equals => {
            if left_type == right_type {
                return Ok(Type::Bool)
            } else {
                return Err("Attempted comparison on expressions of different type");
            }
        }
   }
}

// TODO: planned: symtable: Vec<Variable> arg
pub fn type_expression(expression: Expression ) -> Result<Type, String> {
   
    let typ = match expression {
        Expression::IntLiteral(_) => {Ok(Type::Int)},
        Expression::Variable(_) => {Ok(Type::Int)},         // NOTE: for now! 
        Expression::BinOp{op, left, right } => {
            let left_type = type_expression(*left);
            let right_type = type_expression(*right);
            type_binop(op, left_type, right_type)
        }
        Expression::FuncCall => {
            unimplemented!();
        }
    };
    typ
}
