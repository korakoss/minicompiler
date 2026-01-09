use crate::shared::typing::*;
use crate::stages::common::*;
use crate::stages::ast::*;
use crate::stages::hir::*;
use crate::shared::binops::binop_typecheck;

use std::{collections::HashMap};


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

    fn copy_blank(&self, incr_loop: bool) -> Scope {
        let mut nest_level = self.loop_nest_level;
        if incr_loop {
            nest_level = nest_level + 1;
        }
        Scope {
            scope_vars: HashMap::new(),
            loop_nest_level: nest_level,
            ambient_ret_type: self.ambient_ret_type.clone()
        }
    }
}


pub struct HIRBuilder {
    scope_stack: Vec<Scope>,
    curr_variable_coll: HashMap<VarId, Variable>,
    function_map: HashMap<FuncSignature, (FuncId, Type)>,
    var_counter: usize,
    typetable: TypeTable
}

impl HIRBuilder {
    
    pub fn lower_ast(ast: ASTProgram) -> HIRProgram {
        let ASTProgram{typetable, functions} = ast;

        let function_map: HashMap<FuncSignature, (FuncId, Type)> = functions
            .iter()
            .enumerate()
            .map(|(i, (sgn, func))| (sgn.clone(), (FuncId(i), func.ret_type.clone())))
            .collect();
        let entry = function_map
            .iter()
            .find_map(|(sgn, id)| { (sgn.name == "main")
            .then_some(id)})
            .unwrap().0;

        let mut builder = HIRBuilder {
            scope_stack: Vec::new(),
            curr_variable_coll: HashMap::new(),
            function_map,
            var_counter: 0,
            typetable,
        };

        let mut hir_functions: HashMap<FuncId, HIRFunction> = HashMap::new();

        for (sgn,func) in functions {
            let hir_func = builder.lower_function(func); 
            hir_functions.insert(builder.function_map[&sgn].0, hir_func);
        }

        HIRProgram {
            typetable: builder.typetable, 
            functions: hir_functions,
            entry, 
        }
    }

    fn lower_function(&mut self, func: ASTFunction) -> HIRFunction {
        let ASTFunction { name, args, body, ret_type } = func;
        self.scope_stack.push(Scope::new(ret_type.clone()));
        let arg_ids: Vec<VarId>  = args
            .into_iter()
            .map(|arg| self.register_new_var(Variable{name: arg.0, typ: arg.1}))
            .collect();
        let mut hir_body = self.lower_block(body, false);
        if ret_type == Type::Prim(PrimType::None) {
            hir_body.push(HIRStatement::Return(None));
        }
        let hir_func = HIRFunction { 
            name, 
            args: arg_ids,
            variables: self.curr_variable_coll.clone(),
            body: hir_body, 
            ret_type 
        }; 
        self.curr_variable_coll = HashMap::new();
        self.scope_stack.pop();
        hir_func
    }

    fn lower_lvalue(&self, lvalue: ASTLValue) -> Place {
        match lvalue {
            ASTLValue::Variable(var_name) => {
                let var_id = self.get_var_from_scope(var_name);
                let typ = self.curr_variable_coll[&var_id].typ.clone(); 
                Place {
                    typ,
                    place: PlaceKind::Variable(var_id),
                }
            }
            ASTLValue::FieldAccess { of, field } => {
                let hir_of = self.lower_lvalue(*of);
                let TypeDef::NewType(TypeConstructor::Struct { fields }) = self.typetable.get_typedef(hir_of.typ.clone()) else {
                    panic!("Expression in field access isn't a struct");
                };
                let field_type = fields.get(&field).expect("Field not found in struct").clone();
                Place {
                    typ: field_type,
                    place: PlaceKind::StructField { 
                        of: Box::new(hir_of), 
                        field 
                    }
                }
            }
            ASTLValue::Deref(reference) => {
                let hir_ref = self.lower_expression(reference);
                Place {
                    typ: Type::Reference(Box::new(hir_ref.typ.clone())),
                    place: PlaceKind::Deref(hir_ref)
                }
            }
        }
    }

    fn lower_statement(&mut self, statement: ASTStatement) -> HIRStatement {
        match statement {
            ASTStatement::Let {var, value} => {
                let hir_value = self.lower_expression(value);
                if hir_value.typ != var.typ {
                    panic!("Variable definition inconsistent with value type");
                }
                let var_id = self.register_new_var(var);
                HIRStatement::Let {
                    var: var_id,
                    value: hir_value,
                }
            }
            ASTStatement::Assign { target, value } => {
                let hir_target = self.lower_lvalue(target);
                let hir_value = self.lower_expression(value);
                if hir_target.typ != hir_value.typ {
                    panic!("Non-matching types in assignment to struct field");
                }
                HIRStatement::Assign { target: hir_target, value: hir_value}
            }
            ASTStatement::If { condition, if_body, else_body } => {
                let hir_condition = self.lower_expression(condition);
                if hir_condition.typ != Type::Prim(PrimType::Bool) {
                    panic!("If condition expression not boolean");
                }
                HIRStatement::If {
                    condition: hir_condition, 
                    if_body: self.lower_block(if_body, false), 
                    else_body: match else_body {
                        None => None,
                        Some(block) => Some(self.lower_block(block, false)),
                    }
                }
            }
            ASTStatement::While { condition, body } => {
                let hir_condition = self.lower_expression(condition);
                if hir_condition.typ != Type::Prim(PrimType::Bool) {
                    panic!("If condition expression not boolean");
                }
                HIRStatement::While { 
                    condition: hir_condition,
                    body: self.lower_block(body, true),
                }
            }
            ASTStatement::Break => {
                if self.scope_stack.last().unwrap().loop_nest_level < 1 {
                    panic!("Break statement detected out of loop");
                }
                HIRStatement::Break
            }
            ASTStatement::Continue => {
                if self.scope_stack.last().unwrap().loop_nest_level < 1 {
                    panic!("Continue statement detected out of loop");
                }
                HIRStatement::Continue
            }
            ASTStatement::Return(expr) => {
                let hir_expr = self.lower_expression(expr);
                if hir_expr.typ != self.scope_stack.last().unwrap().ambient_ret_type {
                    panic!("Return statement has unexpected type");
                }
                HIRStatement::Return(Some(hir_expr))
            }
            ASTStatement::Print(expr) => {
                let hir_expr = self.lower_expression(expr);
                HIRStatement::Print(hir_expr)    // Subtler later
            }
        }
    }

    fn lower_block(&mut self, stmts: Vec<ASTStatement>, loop_block: bool) -> Vec<HIRStatement>{
        self.scope_stack.push(self.scope_stack.last().unwrap().copy_blank(loop_block));
        let stmts: Vec<HIRStatement> = stmts 
            .into_iter()
            .map(|stmt| self.lower_statement(stmt))
            .collect();
        self.scope_stack.pop().unwrap();
        stmts
    }

    fn register_new_var(&mut self, var: Variable) -> VarId {
        let var_id = VarId(self.var_counter);
        self.var_counter = self.var_counter + 1;
        self.curr_variable_coll.insert(var_id, var.clone());
        self.scope_stack.last_mut().unwrap().scope_vars.insert(var.name, var_id);
        var_id
    }
    
    fn get_var_from_scope(&self, varname: String) -> VarId {
        // TODO: do it like a normal person
          // union the scopestacks' scopevars, get varname, return from variables

        for scope in self.scope_stack.iter() {
            if let Some(var_id) = scope.scope_vars.get(&varname) {
                return *var_id;
            }
        }
        panic!("Variable name not found in scope");
    }

    fn lower_expression(&self, expr: ASTExpression) -> HIRExpression {
        match expr {
            ASTExpression::IntLiteral(num) => HIRExpression {
                typ: Type::Prim(PrimType::Integer),
                expr: HIRExpressionKind::IntLiteral(num),
            },
            ASTExpression::Variable(varname) => {
                let var_id = self.get_var_from_scope(varname);
                let vartype = self.curr_variable_coll[&var_id].typ.clone();
                HIRExpression {
                    typ: vartype,
                    expr: HIRExpressionKind::Variable(var_id)
                }
            }
            ASTExpression::BinOp{ op, left, right} => {
                let left_hir = self.lower_expression(*left);
                let right_hir = self.lower_expression(*right);
                let result_type = binop_typecheck(&op, &left_hir.typ, &right_hir.typ)
                    .expect("Binop typecheck failed");
                HIRExpression {
                    typ: result_type,
                    expr: HIRExpressionKind::BinOp{ 
                        op, 
                        left: Box::new(left_hir),
                        right: Box::new(right_hir),
                    }
                }
            }
            ASTExpression::FuncCall { funcname, args } => {
                let hir_args: Vec<HIRExpression> = args
                    .into_iter()
                    .map(|arg| self.lower_expression(arg))
                    .collect();
                
                let func_sgn = FuncSignature {
                    name: funcname,
                    argtypes: hir_args
                        .iter()
                        .map(|arg| arg.typ.clone())
                        .collect()
                };
                let (func_id, ret_typ) = &self.function_map[&func_sgn];
                HIRExpression {
                    typ: ret_typ.clone(),
                    expr: HIRExpressionKind::FuncCall{ 
                        id: *func_id, 
                        args: hir_args
                    }
                } 
            }
            ASTExpression::BoolTrue => HIRExpression {
                typ: Type::Prim(PrimType::Bool),
                expr: HIRExpressionKind::BoolTrue,
            },
            ASTExpression::BoolFalse => HIRExpression {
                typ: Type::Prim(PrimType::Bool),
                expr: HIRExpressionKind::BoolFalse,
            },
            ASTExpression::FieldAccess{expr, field} => {
                let hir_expr = self.lower_expression(*expr);
                let TypeDef::NewType(TypeConstructor::Struct { fields }) = self.typetable.get_typedef(hir_expr.typ.clone()) else {
                    panic!("Expression in field access isn't a struct");
                };
                let field_type = fields.get(&field)
                    .expect("Struct in field access doesn't have the requested field")
                    .clone();
                HIRExpression {
                    typ: field_type,
                    expr: HIRExpressionKind::FieldAccess{  
                        expr: Box::new(hir_expr),
                        field 
                     }
                }
            }
            ASTExpression::StructLiteral{typ, fields} => {
                let hir_fields: HashMap<String, HIRExpression> = fields 
                        .into_iter()
                        .map(|(fname, fexpr)| (fname, self.lower_expression(fexpr)))
                        .collect();
                self.typecheck_struct(typ.clone(), hir_fields.clone());
                HIRExpression {
                    typ,
                    expr: HIRExpressionKind::StructLiteral { 
                        fields: hir_fields
                    }
                }
            }
            ASTExpression::Reference(refd) => {
                let hir_refd = self.lower_expression(*refd);
                HIRExpression{
                    typ: Type::Reference(Box::new(hir_refd.typ.clone())),
                    expr: HIRExpressionKind::Reference(Box::new(hir_refd)),
                }
            }
            ASTExpression::Dereference(derefd) => {
                let hir_derefd = self.lower_expression(*derefd);
                let Type::Reference(deref_typ) = hir_derefd.typ.clone() else {
                    unreachable!();
                };
                HIRExpression{
                    typ: Type::Reference(Box::new(*deref_typ)),
                    expr: HIRExpressionKind::Reference(Box::new(hir_derefd)),
                }
            }

        }
    }

    fn typecheck_struct(&self, typ:Type , field_exprs: HashMap<String, HIRExpression>) {
        let TypeDef::NewType(TypeConstructor::Struct { fields: expected_fields }) = self.typetable.get_typedef(typ) else {
            panic!("Type annotation doesn't correspond to a struct newtype");
        };
        for (fname, exp_type) in expected_fields {
            if exp_type != field_exprs.get(&fname).expect("Field not found").typ {
                panic!("Field type doesn't match expected type");
            }
        }
    }
}


