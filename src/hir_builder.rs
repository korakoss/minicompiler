use crate::ast::*;
use crate::hir::*;
use crate::common::*;
use std::{collections::HashMap};


pub struct HIRBuilder {
    pub hir_program: HIRProgram,
}


impl HIRBuilder {
    
    pub fn new() -> HIRBuilder {
        HIRBuilder {
            hir_program: HIRProgram::new(),
        }
    }

    pub fn lower_ast(mut self, ast: ASTProgram) -> HIRProgram {
        let ASTProgram{functions, main_statements} = ast;
        for func in functions {
            self.lower_function(func);
        }
        let globscope = Scope{
            parent_id: None,
            scope_vars: HashMap::new(),
            within_loop: false,
            statements: Vec::new(),
        };
        let globscope_id = self.hir_program.add_scope(globscope);
        for stmt in main_statements {
            self.lower_statement(stmt, &globscope_id);
        }
        self.hir_program.global_scope = Some(globscope_id);
        self.hir_program 
    }
    
        
    fn lower_function(&mut self, function: ASTFunction) {
        let ASTFunction{name, args, body, ret_type} = function;
        let body_scope = Scope{
            parent_id: None,
            scope_vars: HashMap::new(),
            within_loop: false,
            statements: Vec::new(),
        };
        let scope_id = self.hir_program.add_scope(body_scope.clone());
        self.hir_program.add_func(name, HIRFunction{args:args.clone(), body: scope_id.clone(), ret_type: ret_type.clone()});
        for arg in args.clone() {
            self.hir_program.add_var(arg, &scope_id);
        }
        for stmt in body {
            self.lower_statement(stmt, &scope_id);
        }
    }


    fn lower_statement(&mut self, statement: ASTStatement, scope_id: &ScopeId) { 
        let scope = self.hir_program.scopes.get(scope_id).unwrap();
        let hir_statement = match statement {
            ASTStatement::Let{var, value} => {
                let hir_val = self.lower_expression(value, scope_id);
                let varid = self.hir_program.add_var(var, scope_id); // TODO: error handling
                HIRStatement::Let { 
                    var: Place::Variable(varid), 
                    value: hir_val, 
                }
            },
            ASTStatement::Assign { target, value } => {
                match target {
                    ASTExpression::Variable(var_name) => {
                        let var_id = self.hir_program.get_varid(&var_name, scope_id).expect("Unrecognized variable name in scope");
                        let variable = self.hir_program.variables.get(&var_id).unwrap();

                        let hir_expr = self.lower_expression(value, scope_id);

                        if hir_expr.typ != variable.typ {
                            panic!("Attempted value assignment with non-matching types");
                        }

                        HIRStatement::Assign { 
                            target: Place::Variable(var_id), 
                            value: hir_expr
                        }
                    },
                    _ => {panic!("Invalid assignment target");}             // NOTE: eve
                }
            },
            ASTStatement::If { condition, if_body, else_body } => {
                let hir_condition = self.lower_expression(condition, scope_id);
                if hir_condition.typ != Type::Bool {
                    panic!("If condition expression not boolean");
                }
                let hir_if_body = self.transform_block(if_body, scope_id, false); 
                let hir_else_body = match else_body {
                    None => None,
                    Some(stmts) => {
                        Some(self.transform_block(stmts, scope_id, false))
                    }
                };
                HIRStatement::If { condition: hir_condition, if_body: hir_if_body, else_body: hir_else_body}
            },
            ASTStatement::While { condition, body } => {
                let hir_condition = self.lower_expression(condition, scope_id);
                if hir_condition.typ != Type::Bool {
                    panic!("If condition expression not boolean");
                }
                let hir_body = self.transform_block(body, scope_id, true); 
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
                let hir_expr = self.lower_expression(expr, scope_id);
                //let ret_type= self.hir_program.get_scope_ret_type(scope_id).expect("Return statement detected ioutside of a function");
                //if ret_type != hir_expr.typ {
                //    panic!("Type of return expression doesn't match function signature");
                //}
                HIRStatement::Return(hir_expr)
            },
            ASTStatement::Print(expr) => {
                // TODO: this needs subtler type shit later
                let hir_expr = self.lower_expression(expr, scope_id);
                HIRStatement::Print(hir_expr)
            }       
        };
        self.hir_program.scopes.get_mut(scope_id).unwrap().statements.push(hir_statement);
    }
    
    // Split into two funcs?
    fn transform_block(&mut self, statements: Vec<ASTStatement>, parent_scope: &ScopeId, is_loop_body: bool) -> ScopeId{
        let mut block_scope = self.hir_program.scopes.get(parent_scope).unwrap().clone();
        block_scope.parent_id = Some(parent_scope.clone());
        block_scope.statements = Vec::new();
        block_scope.within_loop = block_scope.within_loop || is_loop_body;
        let block_scope_id = self.hir_program.add_scope(block_scope);
        self.hir_program.scopetree.get_mut(&block_scope_id).unwrap().push(parent_scope.clone());
        for stmt in statements {
            self.lower_statement(stmt, &block_scope_id);
        }           
        block_scope_id 
    }

    fn lower_expression(&self, expression: ASTExpression, scope_id: &ScopeId) -> HIRExpression {
        match expression {
            ASTExpression::IntLiteral(num) => {
                HIRExpression{
                    typ: Type::Integer,
                    expr: HIRExpressionKind::IntLiteral(num),
                }
            },
            ASTExpression::Variable(name) => {
                if let Some(varid) = self.hir_program.get_varid(&name, scope_id) {
                    let var = self.hir_program.variables.get(&varid).unwrap();
                    HIRExpression {
                        typ: var.typ.clone(),
                        expr: HIRExpressionKind::Variable(varid), 
                    }
                } else {
                    panic!("Cannot find variable {} in scope", name);
                }
            },
            ASTExpression::BinOp {op, left, right} => {
                let left_hir = self.lower_expression(*left, scope_id);
                let right_hir = self.lower_expression(*right, scope_id);
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
                let hir_args: Vec<HIRExpression> = args.into_iter().map(|x| self.lower_expression(*x, scope_id)).collect();
                let argtypes: Vec<Type> = hir_args.iter().map(|x| x.typ.clone()).collect();
                let funcid = self.hir_program.get_funcid_from_signature(funcname, argtypes).expect("Function with name and signature not found");
                let func_info = self.hir_program.functions.get(funcid).unwrap();
                HIRExpression {
                    typ: func_info.ret_type.clone(),
                    expr: HIRExpressionKind::FuncCall { funcid: funcid.clone(), args: hir_args},
                }
            }
        }
    }
}
