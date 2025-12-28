
use std::collections::BTreeMap;
use std::collections::{HashMap, VecDeque};

use crate::common::*;
use crate::uast::*;
use crate::tast::*;


pub fn type_ast(uast: UASTProgram) -> TASTProgram {
    let UASTProgram{type_defs, functions} = uast;
    let typetable = convert_newtype_defs(type_defs);
    TASTProgram { 
        struct_defs: typetable.clone(),
        functions: functions.into_iter().map(|func| convert_function(func, &typetable)).collect()
    }
}


fn convert_newtype_defs(newtype_defs: HashMap<TypeIdentifier, DeferredNewType>) -> HashMap<TypeIdentifier, NewType> {

    let dep_graph = build_newtype_depgraph(newtype_defs.clone());
    let topo_order = toposort_depgraph(&dep_graph);

    let mut result: HashMap<TypeIdentifier, NewType> = HashMap::new();

    for type_id in topo_order { 
        let newtype = newtype_defs.get(&type_id).unwrap();
        match newtype {
            DeferredNewType::Struct { fields } => {
                result.insert(type_id, convert_struct_typedef(fields.clone(), &result));
            }
        }
    }
    result
}

fn convert_struct_typedef(fields: HashMap<String, DeferredType>, prev_types_map: &HashMap<TypeIdentifier, NewType>) -> NewType {
    let mut tfields : BTreeMap<String, Type> = BTreeMap::new();
    for (fname, ftype) in fields {
        let actual_type = match ftype {
            DeferredType::Resolved(typ) => typ,
            DeferredType::Unresolved(type_id) => Type::NewType(prev_types_map[&type_id].clone()),
        };
        tfields.insert(fname, actual_type);
    }
    NewType::Struct { 
        fields: tfields
    }
}

fn build_newtype_depgraph(newtype_defs: HashMap<TypeIdentifier, DeferredNewType>) -> HashMap<TypeIdentifier, Vec<TypeIdentifier>> {
    let all_def_ids: Vec<TypeIdentifier> = newtype_defs.keys().cloned().collect();
    let dep_graph: HashMap<TypeIdentifier, Vec<TypeIdentifier>> = newtype_defs.iter().map(|(id, def)| (id.clone(), extract_deps(def, all_def_ids.clone()))).collect();
    dep_graph 
}

fn extract_deps(struct_def: &DeferredNewType, all_defn_ids: Vec<TypeIdentifier>) -> Vec<TypeIdentifier> {
    let mut deps = Vec::new();
    match struct_def {
        DeferredNewType::Struct { fields } => {
            for (_, field_type) in fields.iter() {
                match field_type {
                    DeferredType::Resolved(_) => {continue;},
                    DeferredType::Unresolved(type_id) => {
                        if !all_defn_ids.contains(type_id) {
                            panic!("Unrecognized type name");
                        }
                        deps.push(type_id.clone());
                    }
                }
            }
            deps
        }
    }
}




fn convert_function(func: UASTFunction, newtype_table: &HashMap<TypeIdentifier, NewType> ) -> TASTFunction {
    let UASTFunction{name, args, body, ret_type} = func;
    let targs: Vec<Variable> = args.into_iter().map(|t| convert_var(t, &newtype_table)).collect();
    let tbody: Vec<TASTStatement> = body.into_iter().map(|stmt| convert_stmt(stmt, newtype_table)).collect();
    let tret = convert_uast_type(ret_type, newtype_table); 
    TASTFunction {
        name,
        args: targs,
        body: tbody,
        ret_type: tret,
    }
}

fn convert_stmt(stmt: UASTStatement, typetable: &HashMap<TypeIdentifier, NewType>) -> TASTStatement{
    match stmt {
        UASTStatement::Let{ var, value } => {
            TASTStatement::Let {
                var: convert_var(var, &typetable),
                value: convert_expr(value, typetable),
            }
        }
        UASTStatement::Assign { target, value } => {
            TASTStatement::Assign {
                target: convert_expr(target, typetable),
                value: convert_expr(value, typetable)
            }
        }
        UASTStatement::If { condition, if_body, else_body } => {
            let tast_else_body = match else_body {
                Some(stmts) => {Some(stmts.into_iter().map(|stmt| convert_stmt(stmt, typetable)).collect())},
                None => None
            };
            TASTStatement::If {
                condition: convert_expr(condition, typetable),
                if_body: if_body.into_iter().map(|stmt| convert_stmt(stmt, typetable)).collect(),
                else_body:  tast_else_body,
            }
        }
        UASTStatement::While {condition, body} => {
            TASTStatement::While { 
                condition: convert_expr(condition, typetable), 
                    body: body.into_iter().map(|stmt| convert_stmt(stmt, typetable)).collect()
            }
        }
        UASTStatement::Break => TASTStatement::Break,
        UASTStatement::Continue => TASTStatement::Continue,
        UASTStatement::Return(uexpr) => {
            let texpr = convert_expr(uexpr, typetable);
            TASTStatement::Return(texpr)
        }
        UASTStatement::Print(uexpr) => {
            let texpr = convert_expr(uexpr, typetable);
            TASTStatement::Print(texpr)
        }
    }
}

fn convert_expr(uexpr: UASTExpression, typetable: &HashMap<TypeIdentifier, NewType>) -> TASTExpression {
    match uexpr {
        UASTExpression::IntLiteral(num) => TASTExpression::IntLiteral(num),
        UASTExpression::Variable(name) => TASTExpression::Variable(name),
        UASTExpression::BoolTrue => TASTExpression::BoolTrue,
        UASTExpression::BoolFalse => TASTExpression::BoolFalse,
        UASTExpression::BinOp { op, left, right } => {
            TASTExpression::BinOp {
                op,
                left: Box::new(convert_expr(*left, typetable)),
                right: Box::new(convert_expr(*right, typetable)),
            }
        }
        UASTExpression::FuncCall { funcname, args } => {
            TASTExpression::FuncCall { 
                funcname, 
                args: args.into_iter().map(|arg| Box::new(convert_expr(*arg, typetable))).collect(),
            }
        }
        UASTExpression::FieldAccess { expr, field } => {
            TASTExpression::FieldAccess { 
                expr: Box::new(convert_expr(*expr, typetable)), 
                field
            }
        }
        UASTExpression::StructLiteral { fields } => {
            TASTExpression::StructLiteral { 
                fields: fields.into_iter().map(|(fname, fexpr)| (fname, convert_expr(fexpr, typetable))).collect()
            }
        }
    }
}

fn convert_uast_type(utype: DeferredType, newtype_table: &HashMap<TypeIdentifier, NewType>) -> Type {
    match utype {
        DeferredType::Resolved(typ) => typ,
        DeferredType::Unresolved(type_id) => {
            Type::NewType(newtype_table[&type_id].clone())
        }
    }
}



// Clean parts from here on !

fn toposort_depgraph(depgraph: &HashMap<TypeIdentifier, Vec<TypeIdentifier>>) -> Vec<TypeIdentifier> {
    let mut indegrees: HashMap<TypeIdentifier, usize> = HashMap::new();

    for node in depgraph.keys() {
        indegrees.entry(node.clone()).or_insert(0);
    }
    
    for neighbors in depgraph.values().clone() {
        for neighbor in neighbors {
            if let Some(count) = indegrees.get_mut(neighbor) {
                *count += 1;
            }
        }
    }
    
    let mut queue: VecDeque<TypeIdentifier> = indegrees
        .iter()
        .filter(|(_, deg)| **deg == 0)
        .map(|(node, _)| node.clone())
        .collect();
    
    let mut result = Vec::new();
    
    while let Some(node) = queue.pop_front() {
        result.push(node.clone());
        
        if let Some(neighbors) = depgraph.get(&node) {
            for neighbor in neighbors {
                if let Some(deg) = indegrees.get_mut(neighbor) {
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push_back(neighbor.clone());
                    }
                }
            }
        }
    }
    
    if result.len() == indegrees.len() {
        result.reverse();
        result
    } else {
        panic!("Cycle detected in type definitions");
    }
} 



fn convert_var(var: DeferredTypeVariable, newtype_table: &HashMap<TypeIdentifier, NewType> ) -> Variable {
    let DeferredTypeVariable{name, retar_type} = var; 
    let var_type = match retar_type {
        DeferredType::Resolved(typ) => typ,
        DeferredType::Unresolved(type_id) => {
            Type::NewType(newtype_table[&type_id].clone())
        }
    };
    Variable {
        name,
        typ: var_type.clone(),
        size: get_type_size(&var_type),
    }
}

fn get_type_size(typ: &Type) -> usize {
    match typ {
        Type::Integer => 8,
        Type::Bool => 8,
        Type::None => 0,
        Type::NewType(NewType::Struct { fields }) => {
            fields.into_iter().map(|(f_name, f_type)| get_type_size(f_type)).sum()            
        }
    }
}
