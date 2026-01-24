use crate::shared::typing::*;
use crate::shared::tables::*;
use crate::stages::ast::*;
use crate::stages::hir::*;
use crate::shared::binops::binop_typecheck;
use crate::shared::utils::*;

use std::{collections::HashMap};
       
        
pub struct HIRBuilder {
    scope_context: ScopeContext,
    function_map: HashMap<GenericFuncSignature, (FuncId, GenericType)>,
    typetable: GenericTypetable
}

impl HIRBuilder {
    
    pub fn lower_ast(ast: ASTProgram) -> HIRProgram {
        let ASTProgram{typetable, functions} = ast;

        let function_map: HashMap<GenericFuncSignature, (FuncId, GenericType)> = functions
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
            scope_context: ScopeContext::new(),
            function_map,
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
        let ASTFunction { name, typvars, args, body, ret_type } = func;
        self.scope_context.reset(ret_type.clone());
        let arg_ids: Vec<VarId>  = args
            .into_iter()
            .map(|arg| self.scope_context.add_var(Variable { name: arg.0, typ: arg.1}))
            .collect();
        let mut hir_body = self.lower_block(body, false);
        if ret_type == GenericType::Prim(PrimType::None) {
            hir_body.push(HIRStatement::Return(None));
        }
        let hir_func = HIRFunction { 
            name, 
            args: arg_ids,
            variables: self.scope_context.var_map.clone(),
            body: hir_body, 
            ret_type 
        }; 
        self.scope_context.exit_scope();
        hir_func
    }

    fn lower_lvalue(&mut self, lvalue: ASTLValue) -> Place {
        match lvalue {
            ASTLValue::Variable(var_name) => {
                let (id, typ) = self.scope_context.get_var_info(&var_name);
                Place {
                    typ,
                    place: PlaceKind::Variable(id),
                }
            }
            ASTLValue::FieldAccess { of, field: fname } => {
                let hir_of = self.lower_lvalue(*of);
                let GenericType::NewType(id, typvars) = hir_of.typ.clone() else {
                    unreachable!()
                };
                let typdef = self.typetable.bind(id, typvars);
                let GenericShape::Struct{fields} = typdef else {
                    panic!("Expression in field access isn't a struct");
                };
                let field_type = fields.get(&fname).expect("Field not found in struct").clone();
                Place {
                    typ: field_type,
                    place: PlaceKind::StructField { 
                        of: Box::new(hir_of), 
                        field: fname 
                    }
                }
            }
            ASTLValue::Deref(reference) => {
                let hir_ref = self.lower_expression(reference);
                let GenericType::Reference(refd_typ) = hir_ref.typ.clone() else {unreachable!()};
                Place {
                    typ: *refd_typ,
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
                let var_id = self.scope_context.add_var(var);
                HIRStatement::Let {
                    var: var_id,
                    value: hir_value,
                }
            }
            ASTStatement::Assign { target, value } => {
                let hir_target = self.lower_lvalue(target);
                let hir_value = self.lower_expression(value);
                if hir_target.typ != hir_value.typ {
                    panic!("Non-matching types in assignment");
                }
                HIRStatement::Assign { target: hir_target, value: hir_value}
            }
            ASTStatement::If { condition, if_body, else_body } => {
                let hir_condition = self.lower_expression(condition);
                if hir_condition.typ != GenericType::Prim(PrimType::Bool) {
                    panic!("If condition expression not boolean");
                }
                HIRStatement::If {
                    condition: hir_condition, 
                    if_body: self.lower_block(if_body, false), 
                    else_body: else_body.map(|block| self.lower_block(block, false)),
                }
            }
            ASTStatement::While { condition, body } => {
                let hir_condition = self.lower_expression(condition);
                if hir_condition.typ != GenericType::Prim(PrimType::Bool) {
                    panic!("If condition expression not boolean");
                }
                HIRStatement::While { 
                    condition: hir_condition,
                    body: self.lower_block(body, true),
                }
            }
            ASTStatement::Break => {
                if !self.scope_context.in_loop() {
                    panic!("Break statement detected out of loop");
                }
                HIRStatement::Break
            }
            ASTStatement::Continue => {
                if !self.scope_context.in_loop() {
                    panic!("Continue statement detected out of loop");
                }
                HIRStatement::Continue
            }
            ASTStatement::Return(expr) => {
                let hir_expr = self.lower_expression(expr);
                if hir_expr.typ != self.scope_context.ret_type.clone().unwrap() {
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
        self.scope_context.add_scope(loop_block);
        let stmts: Vec<HIRStatement> = stmts 
            .into_iter()
            .map(|stmt| self.lower_statement(stmt))
            .collect();
        self.scope_context.exit_scope();
        stmts
    }

    fn lower_expression(&mut self, expr: ASTExpression) -> HIRExpression {
        match expr {
            ASTExpression::IntLiteral(num) => HIRExpression {
                typ: GenericType::Prim(PrimType::Integer),
                expr: HIRExpressionKind::IntLiteral(num),
            },
            ASTExpression::Variable(varname) => {
                let (id, typ) = self.scope_context.get_var_info(&varname);
                HIRExpression {
                    typ,
                    expr: HIRExpressionKind::Variable(id)
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
                typ: GenericType::Prim(PrimType::Bool),
                expr: HIRExpressionKind::BoolTrue,
            },
            ASTExpression::BoolFalse => HIRExpression {
                typ: GenericType::Prim(PrimType::Bool),
                expr: HIRExpressionKind::BoolFalse,
            },
            ASTExpression::FieldAccess{expr, field} => {
                let hir_expr = self.lower_expression(*expr);
                let GenericType::NewType(id, bindings) = hir_expr.typ.clone() else {
                    panic!("Expression in field access isn't a newtype");
                };
                let expr_typ = self.typetable.bind(id, bindings);
                let GenericShape::Struct { fields } = expr_typ else {
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
                self.typecheck_struct_literal(typ.clone(), hir_fields.clone());
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
                    typ: GenericType::Reference(Box::new(hir_refd.typ.clone())),
                    expr: HIRExpressionKind::Reference(Box::new(hir_refd)),
                }
            }
            ASTExpression::Dereference(derefd) => {
                let hir_derefd = self.lower_expression(*derefd);
                let GenericType::Reference(deref_typ) = hir_derefd.typ.clone() else {
                    unreachable!();
                };
                HIRExpression{
                    typ: *deref_typ,
                    expr: HIRExpressionKind::Dereference(Box::new(hir_derefd)),
                }
            }

        }
    }

    fn typecheck_struct_literal(
        &mut self, 
        typ: GenericType, 
        literal_fields: HashMap<String, HIRExpression>
    ) {
        let GenericType::NewType(id, typvars) = typ.clone() else {unreachable!()};
        let typdef = self.typetable.bind(id, typvars);
        let GenericShape::Struct{fields: expected_fields} = typdef else {
            panic!("Expression in field access isn't a struct");
        };
        for (fname, exp_type) in expected_fields {
            if exp_type != literal_fields.get(&fname).expect("Field not found").typ {
                panic!("Field type doesn't match expected type");
            }
        }
    }
}



struct ScopeContext {
    var_scope_stack: Vec<HashMap<String, VarId>>,
    loop_entrances: Vec<bool>,
    var_map: HashMap<VarId, GenTypeVariable>,
    var_counter: usize,
    ret_type: Option<GenericType>,
}



impl ScopeContext {

    fn new() -> Self {
        ScopeContext {
            var_scope_stack: vec![HashMap::new()],
            loop_entrances: vec![false],
            var_map: HashMap::new(),
            var_counter: 0,
            ret_type: None,
        }
    }

    fn reset(&mut self, new_ret_type: GenericType) {
        self.var_scope_stack = vec![HashMap::new()];
        self.loop_entrances = vec![false];
        self.var_map = HashMap::new();
        self.ret_type = Some(new_ret_type);
    }

    fn add_scope(&mut self, loop_entry: bool) {
        self.var_scope_stack.push(HashMap::new());
        self.loop_entrances.push(loop_entry);
    }

    fn in_loop(&self) -> bool {
        self.loop_entrances.iter().any(|x| *x)
    }

    fn exit_scope(&mut self) {
        self.var_scope_stack.pop();
        self.loop_entrances.pop();
    }
    
    fn add_var(&mut self, var: GenTypeVariable) -> VarId {
        let id = VarId(self.var_counter);
        self.var_counter += 1;
        self.var_scope_stack.last_mut().unwrap().insert(var.name.clone(), id);
        self.var_map.insert(id, var);
        id
    }

    fn get_var_info(&self, name: &String) -> (VarId, GenericType) {
        let id = **self.var_scope_stack
            .iter()
            .flatten()
            .collect::<HashMap<&String, &VarId>>()
            .get(name)
            .expect("Variable name not found in scope");
        (id, self.var_map[&id].typ.clone())
    }
}
 
