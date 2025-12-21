use crate::ast::*;
use crate::hir::*;
use crate::common::*;
use std::{collections::HashMap};


struct HIRBuilder {
    scopes: HashMap<ScopeId, ScopeBlock>,
    variables: HashMap<VarId, VariableInfo>,
    functions: HashMap<FuncId, HIRFunction>,
    function_map: HashMap<(String, Vec<Type>), FuncId>,
    scope_counter: usize,
    var_counter: usize,
    func_counter: usize,
}


impl HIRBuilder {
    
    pub fn add_scope(&mut self, scope: ScopeBlock) -> ScopeId {
        let scope_id = ScopeId(self.scope_counter);
        self.scope_counter = self.scope_counter + 1;
        self.scopes.insert(scope_id, scope);
        scope_id
    }

    pub fn add_var(&mut self, var: VariableInfo, active_scope: &ScopeId) -> VarId {
        if self.get_varid(&var.name, active_scope) != None {
            panic!("Variable name exists in scope");
        }    
        let var_id = VarId(self.var_counter);
        self.scopes.get_mut(active_scope).unwrap().scope_vars.insert(var.name.clone(), var_id.clone());
        self.var_counter = self.var_counter + 1;
        self.variables.insert(var_id, var);
        var_id 
    }

    pub fn add_func(&mut self, func: HIRFunction) -> FuncId {
        let func_id = FuncId(self.func_counter);
        self.func_counter = self.func_counter + 1;
        self.functions.insert(func_id, func);
        func_id
    }

    pub fn get_varid(&self, varname: &String, scope_id: &ScopeId) -> Option<VarId>{
        let scope = self.scopes.get(&scope_id).unwrap();
        if let Some(&varid) = scope.scope_vars.get(varname) {
            Some(varid)
        } else if let Some(parent_id) = scope.parent_id {
            self.get_varid(varname, &parent_id)
        } else {
            None
        }
    } 

    fn transform_function(&mut self, function: Function) {
        let Function{name, args, body, ret_type} = function;
        let body_block = ScopeBlock{
            parent_id: None,
            scope_vars: HashMap::new(),
            within_func: true,
            within_loop: false,
            statements: Vec::new(),
        };
        let scope_id = self.add_scope(body_block.clone());
        for stmt in body {
            self.transform_statement(stmt, &scope_id);
        }
        let func_id = self.add_func(HIRFunction{args:args.clone(), body: body_block, ret_type: ret_type.clone()});
        self.function_map.insert((name, args.into_iter().map(|x| x.typ).collect()), func_id);
    }


    fn transform_statement(&mut self, statement: Statement, active_scope: &ScopeId) { 
        let hir_statement = match statement {
            Statement::Let{var, value} => {
                let hir_val = self.transform_expr(value, active_scope);
                let varid = self.add_var(var, active_scope); // TODO: error handling
                HIRStatement::Let { 
                    var: Place::Variable(varid), 
                    value: hir_val, 
                }
            },
            _ => {unimplemented!();}             
        };
        self.scopes.get_mut(active_scope).unwrap().statements.push(hir_statement);
    }

    fn transform_block(&mut self, statements: Vec<Statement>, parent_scope: &ScopeId) -> ScopeId{
        let mut block_scope = self.scopes.get(parent_scope).unwrap().clone();
        block_scope.statements = Vec::new();
        let block_scope_id = self.add_scope(block_scope);
        for stmt in statements {
            self.transform_statement(stmt, &block_scope_id);
        }           
        block_scope_id 
    }

    fn transform_expr(&self, expr: Expression, active_scope: &ScopeId) -> HIRExpression {
        match expr {
            Expression::IntLiteral(num) => {
                HIRExpression{
                    typ: Type::Integer,
                    expr: TypedExpressionKind::IntLiteral(num),
                }
            },
            Expression::Variable(name) => {
                if let Some(varid) = self.get_varid(&name, active_scope) {
                    let var = self.variables.get(&varid).unwrap();
                    HIRExpression {
                        typ: var.typ.clone(),
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
                let hir_args: Vec<HIRExpression> = args.into_iter().map(|x| self.transform_expr(*x, active_scope)).collect();
                let type_signature: Vec<Type> = hir_args.iter().map(|x| x.typ.clone()).collect();
                let Some(funcid) = self.function_map.get(&(funcname, type_signature)) else {
                    panic!("Function with name and signature not found");
                };
                let func_info = self.functions.get(funcid).unwrap();
                HIRExpression {
                    typ: func_info.ret_type.clone(),
                    expr: TypedExpressionKind::FuncCall { funcid: funcid.clone(), args: hir_args},
                }
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
