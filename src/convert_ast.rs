
use std::collections::BTreeMap;
use std::collections::{HashMap, VecDeque};

use crate::common::*;
use crate::ast::*;


pub struct ASTConverter {
    complete_newtypes: HashMap<TypeIdentifier, CompleteNewType> 
}

impl ASTConverter {
    
    pub fn convert_uast(uast: UASTProgram) -> TASTProgram {
        let UASTProgram{new_types, functions} = uast;
        let converter = ASTConverter::convert_newtype_defs(new_types);

        let mut tfuncs: Vec<TASTFunction> = Vec::new();
        for func in functions {
            let tfunc = converter.convert_function(func);
            tfuncs.push(tfunc);
        }
        TASTProgram { 
            new_types: converter.complete_newtypes, 
            functions:tfuncs,
        }
    }

    fn convert_newtype_defs(newtype_defs: HashMap<TypeIdentifier, DeferredNewType>) -> ASTConverter { 

        let dep_graph = get_newtype_dependencies(&newtype_defs); 
        let topo_order = toposort_depgraph(&dep_graph);

        let mut converter = ASTConverter{
            complete_newtypes: HashMap::new(),
        };
        for type_id in topo_order { 
            let deferred_newtype = newtype_defs[&type_id].clone();
            let complete_newtype = converter.convert_struct_typedef(deferred_newtype);        
            converter.complete_newtypes.insert(type_id, complete_newtype);
        }
        converter
    }

    fn convert_struct_typedef(&mut self, dnt: DeferredNewType) -> CompleteNewType { 
        let DeferredNewType::Struct { fields } = dnt; 
        let mut tfields : BTreeMap<String, Type> = BTreeMap::new();
        for (fname, ftype) in fields {
            let actual_type = match ftype {
                DeferredType::Resolved(typ) => typ,
                DeferredType::Unresolved(type_id) => Type::NewType(self.complete_newtypes[&type_id].clone()),
            };
            tfields.insert(fname, actual_type);
        }
        NewType::Struct { 
            fields: tfields
        }
    }

    fn convert_function(&self, func: UASTFunction) -> TASTFunction {
        let UASTFunction{name, args, body, ret_type} = func;
        TASTFunction {
            name,
            args: args.into_iter().map(|arg| self.convert_var(arg)).collect(),
            body: body.into_iter().map(|stmt| self.convert_statement(stmt)).collect(),
            ret_type: self.convert_type(ret_type)
        }
    }
    
    fn convert_statement(&self, statement: UASTStatement) -> TASTStatement {
        match statement {
            UASTStatement::Let { var, value } => {
                TASTStatement::Let { 
                    var: self.convert_var(var), 
                    value 
                }
            }
            UASTStatement::Assign { target, value } => {
                TASTStatement::Assign { target, value }
            }
            UASTStatement::If { condition, if_body, else_body } => {
                TASTStatement::If { 
                    condition, 
                    if_body: if_body.into_iter().map(|stmt| self.convert_statement(stmt)).collect(),
                    else_body: match else_body {
                        None => None,
                        Some(else_statements) => Some(else_statements.into_iter().map(|stmt| self.convert_statement(stmt)).collect()),
                    }
                }
            }
            UASTStatement::While { condition, body } => {
                TASTStatement::While { 
                    condition, 
                    body: body.into_iter().map(|stmt| self.convert_statement(stmt)).collect(),
                } 
            }
            UASTStatement::Break => TASTStatement::Break,
            UASTStatement::Continue => TASTStatement::Continue,
            UASTStatement::Return(expr) => TASTStatement::Return(expr),
            UASTStatement::Print(expr) => TASTStatement::Print(expr),
        }
    }

    fn convert_var(&self, var: DeferredTypeVariable) -> TypedVariable {
        let DeferredTypeVariable{name, typ} = var; 
        TypedVariable {
            name,
            typ: self.convert_type(typ),         
        }
    }

    fn convert_type(&self, deferred_type: DeferredType) -> Type {
        match deferred_type {
            DeferredType::Resolved(t) => t,
            DeferredType::Unresolved(t_id) => {
                Type::NewType(self.complete_newtypes[&t_id].clone())
            }
        }
    }


}

/*
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
*/


// THE GOOD PLACE


fn get_newtype_dependencies(newtype_defs: &HashMap<TypeIdentifier, DeferredNewType>) -> HashMap<TypeIdentifier, Vec<TypeIdentifier>> {
    let mut dep_graph: HashMap<TypeIdentifier, Vec<TypeIdentifier>> = HashMap::new();
    for (type_id, newtype) in newtype_defs {
        let mut deps: Vec<TypeIdentifier> = Vec::new();
        let DeferredNewType::Struct {fields} = newtype; 
        for (_, field_type) in fields {
            if let DeferredType::Unresolved(dep_id) = field_type {
                deps.push(dep_id.clone());
            }
        }
        dep_graph.insert(type_id.clone(), deps);
    }
    dep_graph
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
