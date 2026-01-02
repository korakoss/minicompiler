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


pub struct HIRBuilder {
    scope_stack: Vec<Scope>,
    new_types: HashMap<TypeIdentifier, DerivType>,
    variables: HashMap<VarId, TypedVariable>,
    curr_variable_coll: Vec<VarId>,
    layouts: LayoutTable,
    function_map: HashMap<CompleteFunctionSignature, FuncId>,
    ret_types: HashMap<FuncId, Type>,
    entry: FuncId,
}

impl HIRBuilder {

    fn new(new_types: &HashMap<TypeIdentifier, DerivType>, functions: &HashMap<FuncSignature<Type>, TASTFunction>) -> HIRBuilder {
        let layouts = LayoutTable::make(new_types.values().cloned().collect());
        
        let mut function_map = HashMap::new();
        let mut ret_types = HashMap::new();
        let mut pot_entry: Option<FuncId> = None;

        for (sgn,func) in functions {
            let id = FuncId(function_map.len());
            function_map.insert(sgn.clone(),id.clone());
            ret_types.insert(id.clone(), func.ret_type.clone());
            if sgn.name == "main" {
                pot_entry = Some(id);
            }
        }
        HIRBuilder {
            scope_stack: Vec::new(),
            new_types: HashMap::new(),
            variables: HashMap::new(),
            curr_variable_coll: Vec::new(),
            layouts,
            function_map,
            ret_types,
            entry: pot_entry.unwrap()
        }
    }

    pub fn lower_ast(tast: TASTProgram) -> HIRProgram {
        let TASTProgram{new_types, functions} = tast;

        let mut builder = HIRBuilder::new(&new_types, &functions);
            
        let mut hir_functions: HashMap<FuncId, HIRFunction> = HashMap::new();

        for (sgn,func) in functions {
            let hir_func = builder.lower_function(func); 
            hir_functions.insert(builder.function_map[&sgn], hir_func);
        }

        HIRProgram {
            functions: hir_functions,
            entry: builder.entry,
            variables: builder.variables,
            layouts: builder.layouts,
        }
    }

    fn lower_function(&mut self, func: TASTFunction) -> HIRFunction {
        let TASTFunction { name, args, body, ret_type } = func;
        self.scope_stack.push(Scope::new(ret_type.clone()));
        let arg_ids: Vec<VarId>  = args
            .into_iter()
            .map(|arg| self.add_variable_in_active_scope(TypedVariable{name: arg.0, typ: arg.1}))
            .collect();
        let hir_body = self.lower_block(body);
        let hir_func = HIRFunction { 
            name, 
            args: arg_ids.clone(),
            body_variables: self.curr_variable_coll.clone(),
            body: hir_body, 
            ret_type 
        }; 
        self.curr_variable_coll = Vec::new();
        self.scope_stack.pop();
        hir_func
    }

    fn lower_statement(&mut self, statement: TASTStatement) -> HIRStatement {
        match statement {
            TASTStatement::Let {var, value} => {
                let hir_value = self.lower_expression(value);
                if self.get_expression_type(hir_value.clone()) != var.typ {
                    panic!("Variable definition inconsistent with value type");
                }
                let var_id = self.add_variable_in_active_scope(var);
                HIRStatement::Let {
                    var: Place::Variable(var_id),
                    value: hir_value,
                }
            }
            TASTStatement::Assign { target, value } => {
                let TASTExpression::Variable(var_name) = target else {
                    panic!("Invalid assignment target");
                };
                let var_id = self.get_var_from_scope(var_name);
                let hir_value = self.lower_expression(value);
                if self.get_expression_type(hir_value.clone()) != self.variables[&var_id].typ {
                    panic!("Type mismatch in assign statement");
                }
                HIRStatement::Assign { 
                    target: Place::Variable(var_id), 
                    value: hir_value
                }
            }
            TASTStatement::If { condition, if_body, else_body } => {
                let hir_condition = self.lower_expression(condition);
                if self.get_expression_type(hir_condition.clone()) != Type::Prim(PrimitiveType::Bool) {
                    panic!("If condition expression not boolean");
                }
                HIRStatement::If {
                    condition: hir_condition, 
                    if_body: self.lower_block(if_body), 
                    else_body: match else_body {
                        None => None,
                        Some(block) => Some(self.lower_block(block)),
                    }
                }
            }
            TASTStatement::While { condition, body } => {
                let hir_condition = self.lower_expression(condition);
                if self.get_expression_type(hir_condition.clone()) != Type::Prim(PrimitiveType::Bool) {
                    panic!("If condition expression not boolean");
                }
                HIRStatement::While { 
                    condition: hir_condition,
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
                let hir_expr = self.lower_expression(expr);
                if self.get_expression_type(hir_expr.clone()) != self.scope_stack.last().unwrap().ambient_ret_type {
                    panic!("Return statement has unexpected type");
                }
                HIRStatement::Return(hir_expr)
            }
            TASTStatement::Print(expr) => {
                let hir_expr = self.lower_expression(expr);
                HIRStatement::Print(hir_expr)    // Subtler later
            }
        }
    }

    fn lower_block(&mut self, stmts: Vec<TASTStatement>) -> Vec<HIRStatement>{
        self.scope_stack.push(self.scope_stack.last().unwrap().copy_blank());
        let stmts: Vec<HIRStatement> = stmts 
            .into_iter()
            .map(|stmt| self.lower_statement(stmt))
            .collect();
        self.scope_stack.pop().unwrap();
        stmts
    }

    fn add_variable_in_active_scope(&mut self, var: TypedVariable) -> VarId {
        let var_id = VarId(self.variables.len());
        self.variables.insert(var_id, var.clone());
        self.scope_stack.last_mut().unwrap().scope_vars.insert(var.name, var_id);
        self.curr_variable_coll.push(var_id.clone());
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

    fn lower_expression(&self, expr: TASTExpression) -> HIRExpression {
        match expr {
            TASTExpression::IntLiteral(num) => HIRExpression::IntLiteral(num),
            TASTExpression::Variable(varname) => {
                let var_id = self.get_var_from_scope(varname);
                HIRExpression::Variable(var_id)
            }
            TASTExpression::BinOp{ op, left, right} => {
                HIRExpression::BinOp{ 
                    op, 
                    left: Box::new(self.lower_expression(*left)), 
                    right: Box::new(self.lower_expression(*right)),  
                }
            }
            TASTExpression::FuncCall { funcname, args } => {
                let hir_args: Vec<HIRExpression> = args
                    .into_iter()
                    .map(|arg| self.lower_expression(arg))
                    .collect();
                
                let func_sgn = FuncSignature {
                    name: funcname,
                    argtypes: hir_args
                        .iter()
                        .map(|arg| self.get_expression_type(arg.clone()).clone())
                        .collect()
                };
                let func_id = self.function_map[&func_sgn].clone();
                HIRExpression::FuncCall{ 
                    id: func_id, 
                    args: hir_args 
                } 
            }
            TASTExpression::BoolTrue => HIRExpression::BoolTrue,
            TASTExpression::BoolFalse => HIRExpression::BoolFalse,
            TASTExpression::FieldAccess{expr, field} => {
                 HIRExpression::FieldAccess{  
                     expr: Box::new(self.lower_expression(*expr)), 
                     field 
                 }
            }
            TASTExpression::StructLiteral{typ, fields} => {
                let hir_fields: HashMap<String, HIRExpression> = fields 
                        .into_iter()
                        .map(|(fname, fexpr)| (fname, self.lower_expression(fexpr)))
                        .collect();
                self.typecheck_struct(typ.clone(), hir_fields.clone());
                HIRExpression::StructLiteral{ 
                    typ, 
                    fields: hir_fields
                }
            }
        }
    }

    fn get_expression_type(&self, expr: HIRExpression) -> Type {
        match expr {
            HIRExpression::IntLiteral(_) => Type::Prim(PrimitiveType::Integer),
            HIRExpression::Variable(var_id) => {
                self.variables[&var_id].typ.clone()
            }
            HIRExpression::BinOp{op, left, right} => {
                binop_typecheck(&op, &self.get_expression_type(*left), &self.get_expression_type(*right)).expect("Binop typecheck failed")
            }
            HIRExpression::FuncCall{id , args} => {
                self.ret_types[&id].clone()
            }
            HIRExpression::BoolTrue | HIRExpression::BoolFalse => Type::Prim(PrimitiveType::Bool),
            HIRExpression::FieldAccess{expr, field} => {
                let Type::Derived(DerivType::Struct{fields}) = self.get_expression_type(*expr) else {
                    panic!("Attempted field access on non-struct value");
                };
                fields[&field].clone()
            }
            HIRExpression::StructLiteral{ typ, fields} => {
                self.typecheck_struct(typ.clone(), fields.clone());
                typ 
            }
        }
    }

    fn typecheck_struct(&self, typ:Type , field_exprs: HashMap<String, HIRExpression>) {
        let Type::Derived(TypeConstructor::Struct{fields: struct_fields}) = typ else {
            panic!("Type annotation doesn't correspond to struct type");
        };         

        for (fname, exp_type) in struct_fields {
            let fexpr = field_exprs.get(&fname).expect("Field not found").clone();
            if self.get_expression_type(fexpr) != exp_type {
                panic!("Field type doesn't match expected type");
            }
        }
    }
}


