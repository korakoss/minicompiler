use crate::tast::*;
use crate::hir::*;
use crate::common::*;
use std::{collections::HashMap};


    
pub fn lower_ast(ast: TASTProgram) -> HIRProgram {
    let TASTProgram{functions, struct_defs} = ast;

    let signature_map: HashMap<FuncSignature, (FuncId, Type)> = functions.iter().enumerate().map(|(i,f)| (f.get_signature(), (FuncId(i), f.ret_type.clone()))).collect();
    
    let mut hir_functions: HashMap<FuncId, HIRFunction> = HashMap::new();
    // Second pass: lower the functions one by one
    let mut entry: Option<FuncId> = None;
    for func in functions {
        let sgn = func.get_signature();
        let id = signature_map.get(&sgn).unwrap().clone().0;
        let hir_func = HIRFunctionBuilder::lower_function(func.clone(), signature_map.clone());
        hir_functions.insert(id, hir_func);
        if func.name == "main" {
            entry = Some(id);
        }
    }
    HIRProgram{ functions: hir_functions, entry: entry}
}



struct HIRFunctionBuilder {
    signature_map: HashMap<FuncSignature, (FuncId, Type)>,
    variables: HashMap<VarId, Variable>,
    ret_type: Type,
    scope_var_stack: Vec<HashMap<String, VarId>>,
    loop_nest_level: usize,
}

impl HIRFunctionBuilder {
    
    fn lower_function(function: TASTFunction, signature_map: HashMap<FuncSignature, (FuncId, Type)>) -> HIRFunction {
        let TASTFunction{name, args, body, ret_type} = function;
       
        let mut builder = HIRFunctionBuilder {
            signature_map: signature_map,
            variables: HashMap::new(),
            ret_type: ret_type.clone(),
            scope_var_stack: Vec::new(),
            loop_nest_level: 0,
        };
        builder.scope_var_stack.push(HashMap::new());
        let arg_ids = args.into_iter().map(|arg| builder.add_var_in_scope(arg)).collect();
        let hir_body = body.into_iter().map(|ast_stmt| builder.lower_statement(ast_stmt)).collect();     
        HIRFunction {
            args: arg_ids,
            body: hir_body,
            variables: builder.variables,
            ret_type,
        }
    }


    fn lower_statement(&mut self, statement: TASTStatement) -> HIRStatement { 
        let hir_statement = match statement {
            TASTStatement::Let{var, value} => {
                let hir_val = self.lower_expression(value);
                let varid = self.add_var_in_scope(var);
                HIRStatement::Let { 
                    var: Place::Variable(varid), 
                    value: hir_val, 
                }
            },
            TASTStatement::Assign { target, value } => {
                match target {
                    TASTExpression::Variable(var_name) => {
                        let var_id = self.get_var_in_scope(&var_name);
                        let hir_expr = self.lower_expression(value);

                        if hir_expr.typ != self.variables.get(&var_id).unwrap().typ {
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
            TASTStatement::If { condition, if_body, else_body } => {
                let hir_condition = self.lower_expression(condition);
                if hir_condition.typ != Type::Bool {
                    panic!("If condition expression not boolean");
                }
                let hir_if_body = self.lower_block(if_body); 
                let hir_else_body = match else_body {
                    None => None,
                    Some(stmts) => {
                        Some(self.lower_block(stmts))
                    }
                };
                HIRStatement::If { condition: hir_condition, if_body: hir_if_body, else_body: hir_else_body}
            },
            TASTStatement::While { condition, body } => {
                let hir_condition = self.lower_expression(condition);
                if hir_condition.typ != Type::Bool {
                    panic!("If condition expression not boolean");
                }
                let hir_body = self.lower_block(body); 
                HIRStatement::While { condition: hir_condition, body: hir_body}
            },
            TASTStatement::Break => {
                if self.loop_nest_level < 1{
                    panic!("Break statement detected out of loop");
                }
                HIRStatement::Break
            },
            TASTStatement::Continue => {
                if self.loop_nest_level < 1{
                    panic!("Break statement detected out of loop");
                }
                HIRStatement::Continue
            },
            TASTStatement::Return(expr) => {
                let hir_expr = self.lower_expression(expr);
                if hir_expr.typ != self.ret_type {
                    panic!("Type of return expression doesn't match function signature");
                }
                HIRStatement::Return(hir_expr)
            },
            TASTStatement::Print(expr) => {
                // TODO: this needs subtler type shit later
                let hir_expr = self.lower_expression(expr);
                HIRStatement::Print(hir_expr)
            }       
        };
        hir_statement
    }
    
    fn lower_block(&mut self, statements: Vec<TASTStatement>) -> Vec<HIRStatement>{
        self.scope_var_stack.push(HashMap::new());
        let hir_stmts = statements.into_iter().map(|stmt| self.lower_statement(stmt)).collect();
        self.scope_var_stack.pop();
        hir_stmts
    }

    fn lower_expression(&self, expression: TASTExpression) -> HIRExpression {
        match expression {
            TASTExpression::IntLiteral(num) => {
                HIRExpression{
                    typ: Type::Integer,
                    expr: HIRExpressionKind::IntLiteral(num),
                }
            },
            TASTExpression::Variable(name) => {
                let var_id = self.get_var_in_scope(&name);
                let var_type = self.variables.get(&var_id).unwrap().typ.clone();
                HIRExpression {
                    typ: var_type,
                    expr: HIRExpressionKind::Variable(var_id),
                }
            },
            TASTExpression::BinOp {op, left, right} => {
                let left_hir = self.lower_expression(*left);
                let right_hir = self.lower_expression(*right);
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
            TASTExpression::FuncCall {funcname, args} => {
                let hir_args: Vec<HIRExpression> = args.into_iter().map(|x| self.lower_expression(*x)).collect();
                let signature = FuncSignature {
                    name: funcname,
                    argtypes: hir_args.iter().map(|x| x.typ.clone()).collect(),
                };
                let (func_id, ret_type) = self.signature_map.get(&signature).expect("Function with name and signature not found").clone();
                HIRExpression {
                    typ: ret_type,
                    expr: HIRExpressionKind::FuncCall { funcid: func_id.clone(), args: hir_args},
                }
            }
            TASTExpression::BoolTrue => {
                HIRExpression{
                    typ: Type::Bool,
                    expr: HIRExpressionKind::BoolTrue,
                }
            }
            TASTExpression::BoolFalse => {
                HIRExpression{
                    typ: Type::Bool,
                    expr: HIRExpressionKind::BoolFalse,
                }
            }
        }
    }

    fn add_var_in_scope(&mut self, var: Variable) -> VarId {
        let var_id = VarId(self.variables.len());
        self.variables.insert(var_id, var.clone());
        self.scope_var_stack.last_mut().unwrap().insert(var.name, var_id);
        var_id
    }

    fn get_var_in_scope(&self, varname: &String) -> VarId {
        // TODO: some typecheck
        let scope_vars: HashMap<String, VarId> = self.scope_var_stack.clone().into_iter().flatten().collect();
        scope_vars.get(varname).expect("Variable name not found in scope").clone()
    }

}



