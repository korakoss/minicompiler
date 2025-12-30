
use std::collections::BTreeMap;
use std::collections::{HashMap, VecDeque};

use crate::ast::*;
use crate::shared::typing::*;


struct TypeTable {
    complete_newtypes: HashMap<TypeIdentifier, DerivType>,
    old_to_new: HashMap<DeferredDerivType, DerivType>,
}

impl TypeTable {

    fn make(newtype_defs: HashMap<TypeIdentifier, DeferredDerivType>) -> TypeTable { 

        let dep_graph = get_newtype_dependencies(&newtype_defs); 
        let topo_order = toposort_depgraph(&dep_graph);

        let mut complete_newtypes: HashMap<TypeIdentifier, DerivType> = HashMap::new();
        let mut old_to_new = HashMap::new();

        for type_id in topo_order { 
            let deferred_newtype = newtype_defs[&type_id].clone();
            let TypeConstructor::Struct { fields } = deferred_newtype.clone();
            let mut tfields : BTreeMap<String, Type> = BTreeMap::new();
            for (fname, ftype) in fields {
                let actual_type = match ftype {
                    DeferredType::Prim(prim_typ) => Type::Prim(prim_typ),
                    DeferredType::Symbolic(type_id) => Type::Derived(complete_newtypes[&type_id].clone()),
                };
                tfields.insert(fname, actual_type);
            }
            let complete_newtype = TypeConstructor::Struct { fields: tfields};
            complete_newtypes.insert(type_id, complete_newtype.clone());
            old_to_new.insert(deferred_newtype, complete_newtype);
        }
        TypeTable { complete_newtypes, old_to_new } 
    }
    
    fn map(&self, deferred_type: &DeferredDerivType) -> DerivType {
        self.old_to_new[deferred_type].clone()        
    }
}

pub struct ASTConverter {
    typetable: TypeTable, 
}

impl ASTConverter {
    
    pub fn convert_uast(uast: UASTProgram) -> TASTProgram {
        let UASTProgram{new_types, functions} = uast;
        let typetable = TypeTable::make(new_types); 
        
        let converter = ASTConverter{typetable};
        let t_functions = functions
            .into_iter()
            .map( |(sgn, func)| (converter.convert_function_signature(sgn), converter.convert_function(func)))
            .collect();
        TASTProgram { 
            new_types: converter.typetable.complete_newtypes, 
            functions: t_functions 
        }
    }

    fn convert_function_signature(&self, fsgn: DeferredFunctionSignature) -> CompleteFunctionSignature {
        let DeferredFunctionSignature{name, argtypes} = fsgn;
        CompleteFunctionSignature {
            name,
            argtypes: argtypes.into_iter().map( |ftyp| (self.convert_type(ftyp))).collect()
        }
    }

    
    fn convert_function(&self, func: UASTFunction) -> TASTFunction {
        let UASTFunction{name, args, body, ret_type} = func;
        TASTFunction {
            name,
            args: args
                .into_iter()
                .map(|(name, deftyp)| (name, self.convert_type(deftyp)))
                .collect(),
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
            typ: self.convert_type(typ)       
        }
    }

    fn convert_type(&self, typ: DeferredType) -> Type {
        match typ {
            DeferredType::Prim(prim) => Type::Prim(prim),
            DeferredType::Symbolic(type_id) => Type::Derived(self.typetable.complete_newtypes[&type_id].clone()),
        }
    }

    

}

/*
fn get_type_size(typ: &Type) -> usize {
    match typ {
        Type::Integer => 8,
        Type::Bool => 8,
        Type::None => 0,
        Type::DerivType(DerivType::Struct { fields }) => {
            fields.into_iter().map(|(f_name, f_type)| get_type_size(f_type)).sum()            
        }
    }
}
*/





// THE GOOD PLACE


fn get_newtype_dependencies(newtype_defs: &HashMap<TypeIdentifier, DeferredDerivType>) -> HashMap<TypeIdentifier, Vec<TypeIdentifier>> {
    let mut dep_graph: HashMap<TypeIdentifier, Vec<TypeIdentifier>> = HashMap::new();
    for (type_id, newtype) in newtype_defs {
        let mut deps: Vec<TypeIdentifier> = Vec::new();
        let DeferredDerivType::Struct {fields} = newtype; 
        for (_, field_type) in fields {
            if let DeferredType::Symbolic(dep_id) = field_type {
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
