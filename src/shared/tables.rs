use std::{collections::{BTreeMap, HashMap, VecDeque}};
use crate::shared::typing::*;


pub struct TypeTable<I, T> {
    pub topo_order: Vec<I>,
    pub newtype_map: HashMap<I, T>
}

pub type GenericTypeTable = TypeTable<PolyTypeIdentifier, GenericCompositeType>;
pub type ConcreteTypeTable = TypeTable<MonoTypeIdentifier, ConcreteCompositeType>;


impl GenericTypeTable {
    
    pub fn new(defs: HashMap<PolyTypeIdentifier, GenericCompositeType>) -> GenericTypeTable {
        GenericTypeTable {
            topo_order: toposort_depgraph(extract_newtype_dependencies(&defs)),
            newtype_map: defs
        }
    }

    pub fn monomorphize(mut self, mut bindings: HashMap<PolyTypeIdentifier, Vec<Binding>>) -> ConcreteTypeTable {
        let mut monom_types: HashMap<MonoTypeIdentifier, ConcreteCompositeType> = HashMap::new();
        let mut topo_order: Vec<MonoTypeIdentifier> = Vec::new();

        for id in self.topo_order {
            let curr_gen_type = self.newtype_map.remove(&id).unwrap();
            let curr_bindings = bindings.remove(&id).unwrap();

            for bdg in curr_bindings {
                let mono_id = id.bind(bdg);
                let mono_type = curr_gen_type.monomorphize(bdg);
                topo_order.push(mono_id);
                monom_types.insert(mono_id, mono_type);
            }
        }

        ConcreteTypeTable {
            topo_order,
            newtype_map: monom_types
        }

    }
}



fn extract_newtype_dependencies(newtype_defs: &HashMap<PolyTypeIdentifier, GenericCompositeType>) -> HashMap<PolyTypeIdentifier, Vec<PolyTypeIdentifier>> {
    let mut dep_graph: HashMap<PolyTypeIdentifier, Vec<PolyTypeIdentifier>> = HashMap::new();
    for (type_id, newtype) in newtype_defs {
        let deps: Vec<PolyTypeIdentifier> = match newtype {
            CompositeType::Struct {fields} => fields
                .values()
                .filter_map(|ftyp| get_underlying_newtype(ftyp.clone()))
                .collect(),
            CompositeType::Enum { variants } => variants
                .iter()
                .filter_map(|vtyp| get_underlying_newtype(vtyp.clone()))
                .collect(),
        };
        dep_graph.insert(type_id.clone(), deps);
    }
    dep_graph
}


fn get_underlying_newtype(t: Type) -> Option<PolyTypeIdentifier> {
    match t {
        Type::Prim(..) => None,
        Type::NewType(id) => Some(id),
        Type::Reference(typ) => get_underlying_newtype(*typ),
    }
}


fn toposort_depgraph(depgraph: HashMap<PolyTypeIdentifier, Vec<PolyTypeIdentifier>>) -> Vec<PolyTypeIdentifier> {

    let mut indegrees: HashMap<PolyTypeIdentifier, usize> = depgraph
        .keys()
        .map(|k| (k.clone(),0))
        .collect();
    
   for neighbor in depgraph.values().flatten() {
        if let Some(count) = indegrees.get_mut(neighbor) {
            *count += 1;
        }
    }

    let mut queue: VecDeque<PolyTypeIdentifier> = depgraph
        .keys()
        .filter(|node| indegrees[node] == 0)
        .cloned()
        .collect();
    let mut result: Vec<PolyTypeIdentifier> = Vec::new();
    
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


