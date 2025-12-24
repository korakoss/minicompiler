use crate::ast::*;
use crate::hir::*;
use crate::common::*;
use std::{collections::HashMap};


pub struct HIRBuilder {
    hir_program: HIRProgram,
    scope_stack: Vec<ScopeContext>
}

impl HIRBuilder {

    pub fn new() -> Self {
        unimplemented!();
    }
    
    pub fn lower_ast(mut self, ast: ASTProgram) -> HIRProgram {
        let ASTProgram{functions} = ast;

        // First pass: register all the signatures
        for func in &functions {
            self.hir_program.register_signature(func.get_signature());
        }

        // Second pass: lower the functions one by one
        for func in functions {
            let sgn = func.get_signature();
            let hir_func = self.lower_function(func);
            self.hir_program.add_function(sgn, hir_func);
        }

        self.hir_program
    }
        
    fn lower_function(&self, function: ASTFunction) -> HIRFunction {
        let ASTFunction{name, args, body, ret_type} = function;
        let mut hir_function = HIRFunction::new(args, ret_type);
        let func_context = ScopeContext{
            scope_vars: hir_function.variables.iter().map(|(id, var)| (var.name, id)).collect(),
            within_loop: false,
            container_func: &hir_function,
        };

        for ast_stmt in body {
            let hir_stmt = self.lower_statement(ast_stmt, &func_context);
            hir_function.body.push(hir_stmt);
        }
        hir_function
    }


    fn lower_statement(&mut self, statement: ASTStatement, context: &ScopeContext) -> HIRStatement { 
        let hir_statement = match statement {
            ASTStatement::Let{var, value} => {
                let hir_val = self.lower_expression(value, context);
                let varid = context.add_var(var);
                HIRStatement::Let { 
                    var: Place::Variable(varid), 
                    value: hir_val, 
                }
            },
            ASTStatement::Assign { target, value } => {
                match target {
                    ASTExpression::Variable(var_name) => {
                        let var_id = context.scope_vars.get(var_name).expect("Variable name not found in scope");
                        let hir_expr = self.lower_expression(value, context);

                        if hir_expr.typ != context.get_var_type(var_name){
                            panic!("Attempted value assignment with non-matching types");
                        }

                        HIRStatement::Assign { 
                            target: Place::Variable(var_id), 
                            value: hir_expr
                        }
                    },
                    _ => {panic!("Invalid assignment target");}             // NOTE: eventually other lvalues too
                }
            },
            ASTStatement::If { condition, if_body, else_body } => {
                let hir_condition = self.lower_expression(condition, context);
                if hir_condition.typ != Type::Bool {
                    panic!("If condition expression not boolean");
                }
                let hir_if_body = self.transform_block(if_body, context, false); 
                let hir_else_body = match else_body {
                    None => None,
                    Some(stmts) => {
                        Some(self.transform_block(stmts, context, false))
                    }
                };
                HIRStatement::If { condition: hir_condition, if_body: hir_if_body, else_body: hir_else_body}
            },
            ASTStatement::While { condition, body } => {
                let hir_condition = self.lower_expression(condition, context);
                if hir_condition.typ != Type::Bool {
                    panic!("If condition expression not boolean");
                }
                let hir_body = self.transform_block(body, context, true); 
                HIRStatement::While { condition: hir_condition, body: hir_body}
            },
            ASTStatement::Break => {
                if !context.within_loop {
                    panic!("Break statement detected out of loop");
                }
                HIRStatement::Break
            },
            ASTStatement::Continue => {
                if !context.within_loop {
                    panic!("Break statement detected out of loop");
                }
                HIRStatement::Continue
            },
            ASTStatement::Return(expr) => {
                let hir_expr = self.lower_expression(expr, context);
                if hir_expr.typ != context.container_func.ret_type {
                    panic!("Type of return expression doesn't match function signature");
                }
                HIRStatement::Return(hir_expr)
            },
            ASTStatement::Print(expr) => {
                // TODO: this needs subtler type shit later
                let hir_expr = self.lower_expression(expr, context);
                HIRStatement::Print(hir_expr)
            }       
        };
        hir_statement
    }
    
    fn transform_block(&mut self, statements: Vec<ASTStatement>, parent_scope: &ScopeContext, is_loop_body: bool) -> Vec<HIRStatement>{
        let mut block_scope = parent_scope.clone();
        block_scope.within_loop = block_scope.within_loop || is_loop_body;
        statements.into_iter().map(|stmt| self.lower_statement(stmt, block_scope)).collect();
    }

    fn lower_expression(&self, expression: ASTExpression, context: &ScopeContext) -> HIRExpression {
        match expression {
            ASTExpression::IntLiteral(num) => {
                HIRExpression{
                    typ: Type::Integer,
                    expr: HIRExpressionKind::IntLiteral(num),
                }
            },
            ASTExpression::Variable(name) => {
                let var_id = context.scope_vars.get(name).expect("Variable not found in scope");
                HIRExpression {
                    typ: context.get_var_type(name),
                    expr: HIRExpressionKind::Variable(var_id),
                }
            },
            ASTExpression::BinOp {op, left, right} => {
                let left_hir = self.lower_expression(*left, scope_id);
                let right_hir = self.lower_expression(*right, scope_id);
                let inferred_type = binop_typecheck(&op, &left_hir.typ, &right_hir.typ).expect("Binop typecheck error");  
                HIRExpression {
                    typ: inferred_type,
                    expr: HIRExpressionKind::BinOp { 
                        op: op,
                        left: Box::new(left_hir), 
                        right: Box::new(right_hir), 
                    }
                }
            },
            ASTExpression::FuncCall {funcname, args} => {
                let hir_args: Vec<HIRExpression> = args.into_iter().map(|x| self.lower_expression(*x, scope_id)).collect();
                let signature = FuncSignature {
                    name: funcname,
                    argtypes: hir_args.iter().map(|x| x.typ.clone()).collect(),
                };
                let funcid = self.hir_program.signature_map.get(signature).expect("Function with name and signature not found");
                let func_ret_type = self.hir_program.functions.get(funcid).unwrap().ret_type;
                HIRExpression {
                    typ: func_ret_type,
                    expr: HIRExpressionKind::FuncCall { funcid: funcid.clone(), args: hir_args},
                }
            }
            ASTExpression::BoolTrue => {
                HIRExpression{
                    typ: Type::Bool,
                    expr: HIRExpressionKind::BoolTrue,
                }
            }
            ASTExpression::BoolFalse => {
                HIRExpression{
                    typ: Type::Bool,
                    expr: HIRExpressionKind::BoolFalse,
                }
            }
        }
    }
}


#[derive(Clone, Debug)]
struct ScopeContext {      
    scope_vars: HashMap<String, VarId>,
    within_loop: bool,
    container_func: &HIRFunction,
}

impl ScopeContext {

    fn add_var(&mut self, var: Variable) -> VarId {
        let var_id = self.container_func.add_var(var);
        self.scope_vars.insert(var.name, var_id);
        var_id
    }
    
    // TODO: maybe change to ID
    fn get_var_type(&self, varname: String) -> Type {
        let var_id = self.scope_vars.get(varname).expect("Variable name not found in scope");
        let var = self.container_func.variables.get(var_id).unwrap();
        var.typ.clone()
    }
}

