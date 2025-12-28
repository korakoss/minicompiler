
use std::collections::{HashMap, VecDeque};

use crate::common::*;
use crate::uast::*;
use crate::tast::*;


pub fn type_ast(uast: UASTProgram) -> TASTProgram {
    let UASTProgram{struct_defs, functions} = uast;
    let typetable = convert_structdefs(struct_defs);
    TASTProgram { 
        struct_defs: typetable.clone(),
        functions: functions.into_iter().map(|func| convert_function(func, &typetable)).collect()
    }
}


fn convert_function(func: UASTFunction, typetable: &HashMap<TypeIdentifier, Type> ) -> TASTFunction {
    let UASTFunction{name, args, body, ret_type} = func;
    let targs: Vec<Variable> = args.into_iter().map(|t| convert_var(t, typetable)).collect();
    let tbody: Vec<TASTStatement> = body.into_iter().map(|stmt| convert_stmt(stmt, typetable)).collect();
    println!("Typetable: {:?}", typetable);
    let tret = typetable[&ret_type].clone();
    println!("{:?} function typed {:?} from original {:?}", name, tret, ret_type);
    TASTFunction {
        name,
        args: targs,
        body: tbody,
        ret_type: tret,
    }
}

fn convert_var(var: UASTVariable, typetable: &HashMap<TypeIdentifier, Type> ) -> Variable {
    let UASTVariable{name, retar_type} = var; 
 
    println!("{:?}",retar_type);
    println!("{:?}",typetable);
    let var_type = typetable[&retar_type].clone();
    Variable {
        name,
        typ: var_type.clone(),
        size: get_type_size(&var_type, typetable),
    }
}

fn get_type_size(typ: &Type, typetable: &HashMap<TypeIdentifier, Type>) -> usize {
    match typ {
        Type::Integer => 8,
        Type::Bool => 8,
        Type::None => 0,
        Type::Derived{name, typ: derived_type} => {
            match derived_type {
                DerivedType::Struct { fields } => {
                    let fsum: usize = fields.iter().map(|(fname, ftype)| get_type_size(ftype, typetable)).sum();
                    fsum
                }
            }
        }
    }
}


fn convert_stmt(stmt: UASTStatement, typetable: &HashMap<TypeIdentifier, Type>) -> TASTStatement{
    match stmt {
        UASTStatement::Let{ var, value } => {
            TASTStatement::Let {
                var: convert_var(var, typetable),
                value: convert_expr(value, typetable),
            }
        }
        UASTStatement::LetStruct {var,value} => {
            TASTStatement::LetStruct { 
                var: convert_var(var, typetable),
                value: convert_struct(value, typetable) }
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

fn convert_expr(uexpr: UASTExpression, typetable: &HashMap<TypeIdentifier, Type>) -> TASTExpression {
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
    }
}

fn convert_struct(ustr: UASTStruct, typetable: &HashMap<TypeIdentifier, Type>) -> TASTStruct {
    let UASTStruct{retar_type, fields} = ustr;
    TASTStruct {
        typ: retar_type,
        fields: fields.into_iter().map(|(fname, expr)| (fname, convert_expr(expr, typetable))).collect(),
    }
}



fn convert_structdefs(struct_defs: HashMap<TypeIdentifier, UASTStructDef>) -> HashMap<TypeIdentifier, Type> {
    let dep_graph = build_depgraph(struct_defs.clone());
    let topo_order = toposort_depgraph(&dep_graph);
    println!("{:?}", topo_order);
    let mut result: HashMap<TypeIdentifier, Type> = HashMap::new();

    result.insert(INT_ID.clone(), Type::Integer);
    result.insert(BOOL_ID.clone(), Type::Bool);
    result.insert(NONE_ID.clone(), Type::None);

    for type_id in topo_order { 
        println!("{:?}", type_id);
        let stru_def = struct_defs.get(&type_id).unwrap();
        let converted_def = DerivedType::Struct { 
            fields: stru_def.fields
                .iter()
                .map(|(fname, ftype)| (fname.clone(), result[ftype].clone()))
                .collect()
        };
        result.insert(type_id.clone(), Type::Derived { name: type_id, typ: converted_def });
    }
    result
}

fn build_depgraph(struct_defs: HashMap<TypeIdentifier, UASTStructDef>) -> HashMap<TypeIdentifier, Vec<TypeIdentifier>> {
    let all_def_ids: Vec<TypeIdentifier> = struct_defs.keys().cloned().collect();
    let dep_graph: HashMap<TypeIdentifier, Vec<TypeIdentifier>> = struct_defs.iter().map(|(id, def)| (id.clone(), extract_deps(def, all_def_ids.clone()))).collect();
    dep_graph 
}

fn extract_deps(struct_def: &UASTStructDef, all_defn_ids: Vec<TypeIdentifier>) -> Vec<TypeIdentifier> {
    let mut deps = Vec::new();
    for (_, field_type) in struct_def.fields.iter() {
        if is_primitive(&field_type) {
            deps.push(field_type.clone());                 // Branching here so primitives cannot
                                                            // be overritten, TODO: do nicer
        } else if all_defn_ids.contains(&field_type) {
            deps.push(field_type.clone());
        } else {
            panic!("Unrecognized field type");
        }
    }
    deps
}

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

fn is_primitive(id: &TypeIdentifier) -> bool {
        id == &*INT_ID || id == &*BOOL_ID 
}

