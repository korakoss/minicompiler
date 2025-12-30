use crate::ast::*;
use crate::hir::*;
use crate::shared::binops::binop_typecheck;
use std::{collections::HashMap};
use crate::shared::typing::*;


struct Scope {
    scope_vars: HashMap<String, VarId>,
    loop_nest_level: usize,
    ambient_ret_type: Type,
}

impl Scope {
    fn new(ret_type: Type) -> Scope {
        Scope { 
            scope_vars: HashMap::new(),
            loop_nest_level: 0,
            ambient_ret_type: ret_type,
        }
    }

    fn copy_blank(&self) -> Scope {
        Scope {
            scope_vars: HashMap::new(),
            loop_nest_level: self.loop_nest_level,
            ambient_ret_type: self.ambient_ret_type.clone()
        }
    }
}


struct HIRBuilder {
    scope_stack: Vec<Scope>,
    new_types: HashMap<TypeIdentifier, CompleteNewType>,
    variables: HashMap<VarId, TypedVariable>,
}

impl HIRBuilder {

    fn new() -> HIRBuilder {
        HIRBuilder {
            scope_stack: Vec::new(),
            new_types: HashMap::new(),
            variables: HashMap::new(),
        }
    }

    pub fn lower_ast(tast: TASTProgram) -> HIRProgram {
        let TASTProgram{new_types, functions} = tast;

        // TODO: nicer
        let entry = functions.iter().filter(|(fsgn, func)| fsgn.name == "main").last().unwrap().0.clone();
        let mut builder = HIRBuilder::new();

        HIRProgram {
            functions: functions.into_iter().map(|(id, func)| (id, builder.lower_function(func))).collect(),
            entry,
            variables: builder.variables
        }
    }

    fn lower_function(&mut self, func: TASTFunction) -> HIRFunction {
        let TASTFunction { name, args, body, ret_type } = func;
        self.scope_stack.push(Scope::new(ret_type.clone()));
        let arg_ids: Vec<VarId>  = args
            .into_iter()
            .map(|arg| self.add_variable_in_active_scope(TypedVariable{name: arg.0, typ: arg.1}))
            .collect();
        let hir_func = HIRFunction { 
            name, 
            args: arg_ids,
            body: body.into_iter().map(|stmt| self.lower_statement(stmt)).collect(), 
            ret_type 
        }; 
        self.scope_stack.pop();
        hir_func
    }

    fn lower_statement(&mut self, statement: TASTStatement) -> HIRStatement {
        match statement {
            TASTStatement::Let {var, value} => {
                if self.infer_expression_type(value.clone()) != var.typ {
                    panic!("Variable definition inconsistent with value type");
                }
                let var_id = self.add_variable_in_active_scope(var);
                HIRStatement::Let {
                    var: Place::Variable(var_id),
                    value: value,
                }
            }
            TASTStatement::Assign { target, value } => {
                let ASTExpression::Variable(var_name) = target else {
                    panic!("Invalid assignment target");
                };
                let var_id = self.get_var_from_scope(var_name);
                if self.infer_expression_type(value.clone()) != self.variables[&var_id].typ {
                    panic!("Type mismatch in assign statement");
                }
                HIRStatement::Assign { 
                    target: Place::Variable(var_id), 
                    value 
                }
            }
            TASTStatement::If { condition, if_body, else_body } => {
                if self.infer_expression_type(condition.clone()) != Type::Primitive(PrimitiveType::Bool) {
                    panic!("If condition expression not boolean");
                }
                HIRStatement::If {
                    condition, 
                    if_body: self.lower_block(if_body), 
                    else_body: match else_body {
                        None => None,
                        Some(block) => Some(self.lower_block(block)),
                    }
                }
            }
            TASTStatement::While { condition, body } => {
                if self.infer_expression_type(condition.clone()) != Type::Primitive(PrimitiveType::Bool) {
                    panic!("If condition expression not boolean");
                }
                HIRStatement::While { 
                    condition,
                    body: self.lower_block(body),
                }
            }
            TASTStatement::Break => {
                if self.scope_stack.last().unwrap().loop_nest_level < 1 {
                    panic!("Break statement detected out of loop");
                }
                HIRStatement::Break
            }
            TASTStatement::Continue => {
                if self.scope_stack.last().unwrap().loop_nest_level < 1 {
                    panic!("Continue statement detected out of loop");
                }
                HIRStatement::Continue
            }
            TASTStatement::Return(expr) => {
                if self.infer_expression_type(expr.clone()) != self.scope_stack.last().unwrap().ambient_ret_type {
                    panic!("Return statement has unexpected type");
                }
                HIRStatement::Return(expr)
            }
            TASTStatement::Print(expr) => HIRStatement::Print(expr),    // Subtler later
        }
    }

    fn lower_block(&mut self, stmts: Vec<TASTStatement>) -> Vec<HIRStatement> {
        self.scope_stack.push(self.scope_stack.last().unwrap().copy_blank());
        let hir_block: Vec<HIRStatement> = stmts 
            .into_iter()
            .map(|stmt| self.lower_statement(stmt))
            .collect();
        self.scope_stack.pop();
        hir_block 
    }

    fn add_variable_in_active_scope(&mut self, var: TypedVariable) -> VarId {
        let var_id = VarId(self.variables.len());
        self.variables.insert(var_id, var.clone());
        self.scope_stack.last_mut().unwrap().scope_vars.insert(var.name, var_id);
        var_id
    }
    
    fn get_var_from_scope(&self, varname: String) -> VarId {
        // TODO: do it like a normal person
          // union the scopestacks' scopevars, get varname, return from variables

        for scope in self.scope_stack.iter() {
            if let Some(var_id) = scope.scope_vars.get(&varname) {
                return var_id.clone();
            }
        }
        panic!("Variable name not found in scope");
    }

    fn infer_expression_type(&self, expr: ASTExpression) -> Type {
        match expr {
            ASTExpression::IntLiteral(_) => Type::Primitive(PrimitiveType::Integer),            // This solution sucks btw, with the type type syntax
            ASTExpression::Variable(varname) => {
                let var_id = self.get_var_from_scope(varname);
                self.variables[&var_id].typ.clone()
            },
            ASTExpression::BinOp{op, left, right} => binop_typecheck(&op, self.infer_expression_type(*left), self.infer_expression_type(*right)),
            ASTExpression::FuncCall { funcname, args } => {
                unimplemented!();
            },
            ASTExpression::BoolTrue => Type::Primitive(PrimitiveType::Bool),
            ASTExpression::BoolFalse => Type::Primitive(PrimitiveType::Bool),
            ASTExpression::FieldAccess { expr, field } => {
                let Type::NewType(NewType::Struct {fields}) = self.infer_expression_type(*expr) else {
                    panic!("Attempted field access on non-struct value");
                };
                fields[&field]
            }
            ASTExpression::StructLiteral { fields } => {
                unimplemented!();
            }
        }
    }
}
