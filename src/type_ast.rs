
use std::collections::{HashMap, VecDeque};

use crate::common::*;
use crate::uast::*;
use crate::tast::*;


pub fn type_ast(uast: UASTProgram) -> TASTProgram {
    let UASTProgram{struct_defs, functions} = uast;
}


fn convert_function(func: UASTFunction, typetable: HashMap<TypeIdentifier, Type> ) -> TASTFunction {
    let UASTFunction{name, args, body, ret_type} = func;
    let targs: Vec<Variable> = args.into_iter().map(|t| convert_var(t, typetable.clone())).collect();
    let tbody: Vec<TASTStatement> = body.into_iter().map(|stmt| convert_stmt(stmt, typetable.clone())).collect();
    let tret = typetable[&ret_type];
    TASTFunction {
        name,
        args: targs,
        body: tbody,
        ret_type: tret,
    }
}

fn convert_var(var: UASTVariable, typetable: HashMap<TypeIdentifier, Type> ) -> Variable {
   let UASTVariable{name, retar_type} = var; 
   Variable {
        name,
        typ: typetable[&retar_type].clone(),
    }
}

fn convert_stmt(stmt: UASTStatement, typetable: HashMap<TypeIdentifier, Type>) -> TASTStatement{
    unimplemented!()
}

fn convert_expr() {}


fn convert_structdefs(struct_defs: HashMap<TypeIdentifier, UASTStructDef>) -> HashMap<TypeIdentifier, Type> {
    let dep_graph = build_depgraph(struct_defs.clone());
    let topo_order = toposort_depgraph(&dep_graph);
    let mut result: HashMap<TypeIdentifier, Type> = HashMap::new();
    for type_id in topo_order { 
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
    let all_def_ids: Vec<TypeIdentifier> = struct_defs.keys().collect().clone(); 
    let dep_graph: HashMap<TypeIdentifier, Vec<TypeIdentifier>> = struct_defs.iter().map(|(id, def)| (id.clone(), extract_deps(def, all_def_ids))).collect();
    dep_graph 
}

fn extract_deps(struct_def: &UASTStructDef, all_defn_ids: Vec<TypeIdentifier>) -> Vec<TypeIdentifier> {
    let mut deps = Vec::new();
    for (_, field_type) in struct_def.fields.iter() {
        if is_primitive(&field_type) {
             continue;
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
        for neighbor in neighbors.clone() {
            *indegrees.entry(neighbor).or_insert(0) += 1;
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
                let deg = indegrees.get_mut(neighbor).unwrap();
                *deg -= 1;
                if *deg == 0 {
                    queue.push_back(neighbor.clone());
                }
            }
        }
    }
    
    if result.len() == indegrees.len() {
        result
    } else {
        panic!("Cycle detected in type definitions");
    }
} 

fn is_primitive(id: &TypeIdentifier) -> bool {
    let int_id = TypeIdentifier(String::from("int"));   
    let bool_id = TypeIdentifier(String::from("bool"));   
    *id == int_id|| *id == bool_id
}

