use crate::ast::*;
use crate::hir::*;
use crate::common::*;
use std::{collections::HashMap};


pub struct HIRBuilder {
    pub hir: HIRProgram,
    function_map: HashMap<(String, Vec<Type>), FuncId>,
    scope_counter: usize,
    var_counter: usize,
    func_counter: usize,
}


impl HIRBuilder {
    
    pub fn new() -> HIRBuilder {
        HIRBuilder {
            hir: HIRProgram::new(),
            function_map: HashMap::new(),
            scope_counter: 0,
            var_counter: 0,
            func_counter: 0,
        }
    }

    pub fn lower_ast(&mut self, ast: ASTProgram) -> HIRProgram {
        let ASTProgram{functions, main_statements} = ast;
        for func in functions {
            self.transform_function(func);
        }
        let globscope = Scope{
            parent_id: None,
            scope_vars: HashMap::new(),
            within_loop: false,
            within_func: false,
            statements: Vec::new(),
        };
        let globscope_id = self.add_scope(globscope);
        for stmt in main_statements {
            self.transform_statement(stmt, &globscope_id);
        }
        self.hir.global_scope = Some(globscope_id);
        self.hir.clone() 
    }

    fn add_scope(&mut self, scope: Scope) -> ScopeId {
        let scope_id = ScopeId(self.scope_counter);
        self.scope_counter = self.scope_counter + 1;
        self.hir.scopes.insert(scope_id, scope);
        self.hir.scopetree.insert(scope_id, Vec::new());
        scope_id
    }

    fn add_var(&mut self, var: Variable, active_scope: &ScopeId) -> VarId {
        if self.hir.get_varid(&var.name, active_scope) != None {
            panic!("Variable name exists in scope");
        }    
        let var_id = VarId(self.var_counter);
        self.hir.scopes.get_mut(active_scope).unwrap().scope_vars.insert(var.name.clone(), var_id.clone());
        self.var_counter = self.var_counter + 1;
        self.hir.variables.insert(var_id, var);
        var_id 
    }

    fn add_func(&mut self, func: HIRFunction) -> FuncId {
        let func_id = FuncId(self.func_counter);
        self.func_counter = self.func_counter + 1;
        self.hir.functions.insert(func_id, func);
        func_id
    }

    fn transform_function(&mut self, function: ASTFunction) {
        let ASTFunction{name, args, body, ret_type} = function;
        let body_block = Scope{
            parent_id: None,
            scope_vars: HashMap::new(),
            within_func: true,
            within_loop: false,
            statements: Vec::new(),
        };
        let scope_id = self.add_scope(body_block.clone());
        for arg in args.clone() {
            self.add_var(arg, &scope_id);
        }
        for stmt in body {
            self.transform_statement(stmt, &scope_id);
        }
        let func_id = self.add_func(HIRFunction{args:args.clone(), body: scope_id, ret_type: ret_type.clone()});
        self.function_map.insert((name, args.into_iter().map(|x| x.typ).collect()), func_id);
    }


    fn transform_statement(&mut self, statement: ASTStatement, active_scope: &ScopeId) { 
        let scope = self.hir.scopes.get(active_scope).unwrap();
        let hir_statement = match statement {
            ASTStatement::Let{var, value} => {
                let hir_val = self.transform_expr(value, active_scope);
                let varid = self.add_var(var, active_scope); // TODO: error handling
                HIRStatement::Let { 
                    var: Place::Variable(varid), 
                    value: hir_val, 
                }
            },
            ASTStatement::Assign { target, value } => {
                match target {
                    ASTExpression::Variable(varname) => {
                        let varid = self.hir.get_varid(&varname, active_scope).expect("Unrecognized variable name in scope");
                        let hir_expr = self.transform_expr(value, active_scope);
                        // TODO: typecheck!!! IMPORTANT!!!
                        HIRStatement::Assign { 
                            target: Place::Variable(varid), 
                            value: hir_expr
                        }
                    },
                    _ => {panic!("Invalid assignment target");}
                }
            },
            ASTStatement::If { condition, if_body, else_body } => {
                let hir_condition = self.transform_expr(condition, active_scope);
                if hir_condition.typ != Type::Bool {
                    panic!("If condition expression not boolean");
                }
                let hir_if_body = self.transform_block(if_body, active_scope, false); 
                let hir_else_body = match else_body {
                    None => None,
                    Some(stmts) => {
                        Some(self.transform_block(stmts, active_scope, false))
                    }
                };
                HIRStatement::If { condition: hir_condition, if_body: hir_if_body, else_body: hir_else_body}
            },
            ASTStatement::While { condition, body } => {
                let hir_condition = self.transform_expr(condition, active_scope);
                if hir_condition.typ != Type::Bool {
                    panic!("If condition expression not boolean");
                }
                let hir_body = self.transform_block(body, active_scope, true); 
                HIRStatement::While { condition: hir_condition, body: hir_body}
            },
            ASTStatement::Break => {
                if !scope.within_loop {
                    panic!("Break statement detected out of loop");
                }
                HIRStatement::Break
            },
            ASTStatement::Continue => {
                if !scope.within_loop {
                    panic!("Break statement detected out of loop");
                }
                HIRStatement::Continue
            },
            ASTStatement::Return(expr) => {
                // TODO: this needs type checks!!! IMPORTANT!!!
                let hir_expr = self.transform_expr(expr, active_scope);
                HIRStatement::Return(hir_expr)
            },
            ASTStatement::Print(expr) => {
                // TODO: this needs subtler type shit later
                let hir_expr = self.transform_expr(expr, active_scope);
                HIRStatement::Print(hir_expr)
            }       
        };
        self.hir.scopes.get_mut(active_scope).unwrap().statements.push(hir_statement);
    }
    
    // Split into two funcs?
    fn transform_block(&mut self, statements: Vec<ASTStatement>, parent_scope: &ScopeId, is_loop_body: bool) -> ScopeId{
        let mut block_scope = self.hir.scopes.get(parent_scope).unwrap().clone();
        block_scope.parent_id = Some(parent_scope.clone());
        block_scope.statements = Vec::new();
        block_scope.within_loop = block_scope.within_loop || is_loop_body;
        let block_scope_id = self.add_scope(block_scope);
        self.hir.scopetree.get_mut(&block_scope_id).unwrap().push(parent_scope.clone());
        for stmt in statements {
            self.transform_statement(stmt, &block_scope_id);
        }           
        block_scope_id 
    }

    fn transform_expr(&self, expr: ASTExpression, active_scope: &ScopeId) -> HIRExpression {
        match expr {
            ASTExpression::IntLiteral(num) => {
                HIRExpression{
                    typ: Type::Integer,
                    expr: HIRExpressionKind::IntLiteral(num),
                }
            },
            ASTExpression::Variable(name) => {
                if let Some(varid) = self.hir.get_varid(&name, active_scope) {
                    let var = self.hir.variables.get(&varid).unwrap();
                    HIRExpression {
                        typ: var.typ.clone(),
                        expr: HIRExpressionKind::Variable(varid), 
                    }
                } else {
                    panic!("Cannot find variable {} in scope", name);
                }
            },
            ASTExpression::BinOp {op, left, right} => {
                let left_hir = self.transform_expr(*left, active_scope);
                let right_hir = self.transform_expr(*right, active_scope);
                let inferred_type = binop_typecheck(&op, &left_hir.typ, &right_hir.typ);  
                if let Some(typ) = inferred_type {
                    let binop_expr = HIRExpressionKind::BinOp { 
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
            ASTExpression::FuncCall {funcname, args} => {
                let hir_args: Vec<HIRExpression> = args.into_iter().map(|x| self.transform_expr(*x, active_scope)).collect();
                let type_signature: Vec<Type> = hir_args.iter().map(|x| x.typ.clone()).collect();
                let Some(funcid) = self.function_map.get(&(funcname, type_signature)) else {
                    panic!("Function with name and signature not found");
                };
                let func_info = self.hir.functions.get(funcid).unwrap();
                HIRExpression {
                    typ: func_info.ret_type.clone(),
                    expr: HIRExpressionKind::FuncCall { funcid: funcid.clone(), args: hir_args},
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
