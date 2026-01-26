use std::{collections::{BTreeMap, HashMap, VecDeque}, hash::Hash};
use crate::shared::typing::*;


#[cfg(test)]
#[path = "../tests/test_tables.rs"]
mod tests;


#[derive(Debug, Clone)]
pub struct GenericTypetable {
    topo_order: Vec<NewtypeId>,
    monomorphizations: HashMap<NewtypeId, HashMap<Vec<ConcreteType>,ConcreteShape>>, // Change for a HashMap to avoid duplicates
    pub defs: HashMap<NewtypeId, GenericTypeDef>,
}

impl GenericTypetable {

    pub fn new(defs: HashMap<NewtypeId, GenericTypeDef>) -> Self {
        Self { 
            topo_order: toposort_depgraph(extract_newtype_dependencies(&defs)), 
            monomorphizations: defs.keys()
                .map(|id| (id.clone(), HashMap::new()))
                .collect(),
            defs,
        }
    }
    
    pub fn bind(
        &mut self, 
        id: NewtypeId, 
        typ_var_vals: Vec<GenericType>
    ) -> GenericShape {
        let def = self.defs[&id].clone();
        let type_params: BTreeMap<String, GenericType> = def.type_params
            .iter()
            .cloned()
            .zip(typ_var_vals.iter().cloned())
            .collect();
        match def.defn {
            GenericShape::Struct { fields } => {
                GenericShape::Struct { 
                    fields: fields
                        .into_iter()
                        .map(|(name, typ)| (name, typ.bind(&type_params)))
                        .collect()
                }
            }
            GenericShape::Enum {..} => {
                unimplemented!();
            }
        }
    }

    pub fn monomorphize(
        &mut self, 
        id: NewtypeId, 
        typ_var_vals: Vec<ConcreteType>
    ) -> ConcreteShape {
        let def = self.defs[&id].clone();
        let type_params: BTreeMap<String, ConcreteType> = def.type_params
            .iter()
            .cloned()
            .zip(typ_var_vals.iter().cloned())
            .collect();
        let monomorph = match def.defn {
            GenericShape::Struct { fields } => {
                ConcreteShape::Struct { 
                    fields: fields
                        .into_iter()
                        .map(|(name, typ)| (name, typ.monomorphize(&type_params)))
                        .collect()
                }
            }
            GenericShape::Enum {..} => {
                unimplemented!();
            }
        };
        self.monomorphizations.get_mut(&id).unwrap().insert(typ_var_vals,monomorph.clone());
        monomorph
    }

    pub fn get_genericity_rank(
        &mut self,
        typ: &ConcreteType,
    ) -> usize {
        match typ {
            ConcreteType::Prim(..) => 0,
            ConcreteType::Reference(ref_typ) => self.get_genericity_rank(ref_typ) + 1,
            ConcreteType::NewType(id, type_params) => {
                let type_shape = self.monomorphize(id.clone(), type_params.clone());
                match type_shape {
                    ConcreteShape::Struct { fields } => {
                        fields
                            .values()
                            .map(|typ| self.get_genericity_rank(typ))
                            .max()
                            .unwrap() + 1
                    },
                    ConcreteShape::Enum { .. } => {
                        unimplemented!();
                    }
                }
            }
        }
    }
}


pub type GenericTypeDef = NewtypeDef<GenericType>;
pub type GenericShape = NewtypeShape<GenericType>;

pub type ConcreteShape = NewtypeShape<ConcreteType>;
pub type ConcreteTypeDef = NewtypeDef<ConcreteType>;


#[derive(Clone, Debug)]
pub struct NewtypeDef<T> {
    pub type_params: Vec<String>,
    pub defn: NewtypeShape<T>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum NewtypeShape<T>{
    Struct {
        fields: BTreeMap<String, T>
    },
    Enum {
        variants: Vec<T>
    },
}


fn extract_newtype_dependencies(newtype_defs: &HashMap<NewtypeId, GenericTypeDef>) -> HashMap<NewtypeId, Vec<NewtypeId>> {
    let mut dep_graph: HashMap<NewtypeId, Vec<NewtypeId>> = HashMap::new();
    for (type_id, newtype) in newtype_defs {
        let deps: Vec<NewtypeId> = match &newtype.defn {
            NewtypeShape::Struct {fields} => fields
                .values()
                .flat_map(extract_type_id)
                .collect(),
            NewtypeShape::Enum { variants } => variants
                .iter()
                .flat_map(extract_type_id)
                .collect(),
        };
        dep_graph.insert(type_id.clone(), deps);
    }
    dep_graph
}


fn extract_type_id(t: &GenericType) -> Vec<NewtypeId> {
    match t {
        GenericType::Prim(..) => vec![],
        GenericType::NewType(id, t_params) => {
            let mut deps: Vec<NewtypeId> = t_params
                .iter()
                .flat_map(extract_type_id)
                .collect::<Vec<_>>();
            deps.push(id.clone());
            deps
        }
        GenericType::Reference(typ) => extract_type_id(typ),
        GenericType::TypeVar(..) => vec![] 
    }
}


fn toposort_depgraph<T: Clone + Eq + PartialEq + Hash>(depgraph: HashMap<T, Vec<T>>) -> Vec<T> {

    let mut indegrees: HashMap<T, usize> = depgraph
        .keys()
        .map(|k| (k.clone(),0))
        .collect();
    
   for neighbor in depgraph.values().flatten() {
        if let Some(count) = indegrees.get_mut(neighbor) {
            *count += 1;
        }
    }

    let mut queue: VecDeque<T> = depgraph
        .keys()
        .filter(|node| indegrees[node] == 0)
        .cloned()
        .collect();
    let mut result: Vec<T> = Vec::new();
    
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
    if result.len() != indegrees.len() {
        panic!("Cycle detected in type definitions");
    }
    result.reverse();
    result
}
