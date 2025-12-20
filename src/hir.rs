use crate::common::*;
use std::collections::HashMap;
use crate::ast::*;


// AFTER TYPING+VARSCOPING


#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct VarId(pub usize);

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct ScopeId(pub usize);

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct FuncId(pub usize);


enum TypedExpressionKind {
    IntLiteral(i32),
    Variable(VarId),
    BinOp {
        op: BinaryOperator,
        left: Box<HIRExpression>,
        right: Box<HIRExpression>,
    },
}

pub struct HIRExpression {
    pub typ: Type,
    pub expr: TypedExpressionKind,
}


pub enum Place {
    Variable,
}


pub struct ScopeBlock {      
    parent_id: Option<ScopeId>,
    scope_vars: HashMap<String, VarId>,
    within_func: bool,
    within_loop: bool,
    statements: Vec<HIRStatement>,
}


pub enum HIRStatement {
    Let {
        var: Place,     // Expected to be Variable
        value: HIRExpression,
    },
    Assign {
        target: HIRExpression,   // Expected to be l-value
        value: HIRExpression,
    },
    If {
        condition: HIRExpression,    // Expected to be Boolean
        if_body: ScopeBlock,    
        else_body: Option<ScopeBlock>,
    },
    While {
        condition: HIRExpression,
        body: ScopeBlock,
},
    Break,
    Continue,
    Return(HIRExpression),
    Print(HIRExpression),
}




struct HIRBuilder {
   scopes: HashMap<ScopeId, ScopeBlock>,
   variables: HashMap<VarId, VariableInfo>
}


impl HIRBuilder {

    pub fn get_varid(&self, varname: String, scope_id: ScopeId) -> Option<VarId>{
        let scope = self.scopes.get(&scope_id).unwrap();
        if let Some(&varid) = scope.scope_vars.get(&varname) {
            Some(varid)
        } else if let Some(parent_id) = scope.parent_id {
            self.get_varid(varname, parent_id)
        } else {
            None
        }
    } 

    fn transform_expr(&self, expr: Expression, active_scope: ScopeId) -> HIRExpression {
        match expr {
            Expression::IntLiteral(num) => {
                HIRExpression{
                    typ: Type::Integer,
                    expr: TypedExpressionKind::IntLiteral(num),
                }
            },
            Expression::Variable(name) => {
                if let Some(varid) = self.get_varid(name, active_scope) {
                    let var = self.variables.get(varid).unwrap();
                    HIRExpression {
                        typ: var.typ,
                        expr: TypedExpressionKind::Variable(varid), 
                    }
                } else {
                    panic!("Cannot find variable {} in scope", name);
                }
            },
            Expression::BinOp {op, left, right} => {
                let left_hir = self.transform_expr(*left, active_scope);
                let right_hir = self.transform_expr(*right, active_scope);
                let inferred_type = binop_typecheck(&op, &left_hir.typ, &right_hir.typ);  
                if let Some(typ) = inferred_type {
                    let binop_expr = TypedExpressionKind::BinOp { 
                        op, 
                        left: Box::new(left_hir), 
                        right: Box::new(right_hir), 
                    };
                    HIRExpression {
                        typ: typ,
                        expr: binop_expr
                    }
                } else {
                    panic!("Binop typecheck failed");
                }
            },
            Expression::FuncCall {funcname, args} => {
            }
        }
    }
}




fn binop_typecheck(op: &BinaryOperator, left_type: &Type, right_type: &Type) -> Option<Type> {
    
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
