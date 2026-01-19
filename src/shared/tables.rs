use std::{collections::{BTreeMap, HashMap, VecDeque}, hash::Hash};
use crate::shared::typing::*;



#[derive(Debug, Clone)]
pub struct GenericTypetable {
    topo_order: Vec<NewtypeId>,
    pub defs: HashMap<NewtypeId, GenericTypeDef>,
    monomorph_requests: Vec<(NewtypeId, Vec<ConcreteType>)>
}

impl GenericTypetable {

    pub fn new(defs: HashMap<NewtypeId, GenericTypeDef>) -> Self {
        Self { 
            topo_order: toposort_depgraph(extract_newtype_dependencies(&defs)), 
            defs,
            monomorph_requests: Vec::new(),
        }
    }

    pub fn topo_iter(&self) -> impl Iterator<Item = (&NewtypeId, &GenericTypeDef)> {
        self.topo_order.iter().map(|id| (id, &self.defs[&id]))
    }

    pub fn eval(&self, id: NewtypeId, typ_var_vals: Vec<ConcreteType>) -> ConcreteType {
        // Attempts to evaluate a generic newtype with some typing expression substituted, hoping to get a concrete type
        let typedef = self.defs[&id].clone();
        if typedef.type_params.len() != typ_var_vals.len() {
            panic!("Number of supplied type parameters doesn't match the expected number");
        }
        let bindings = Binding(
            typedef.type_params
                .iter()
                .cloned()
                .zip(typ_var_vals.into_iter())
                .collect::<BTreeMap<String, ConcreteType>>()
        );

        unimplemented!(); 

    }
}





fn extract_newtype_dependencies(newtype_defs: &HashMap<NewtypeId, GenericTypeDef>) -> HashMap<NewtypeId, Vec<NewtypeId>> {
    let mut dep_graph: HashMap<NewtypeId, Vec<NewtypeId>> = HashMap::new();
    for (type_id, newtype) in newtype_defs {
        let deps: Vec<NewtypeId> = match &newtype.defn {
            CompositeType::Struct {fields} => fields
                .values()
                .map(|ftyp| extract_type_id(&ftyp))
                .flatten()
                .collect(),
            CompositeType::Enum { variants } => variants
                .iter()
                .map(|vtyp| extract_type_id(&vtyp))
                .flatten()
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
            let mut deps: Vec<NewtypeId> = t_params.iter().map(|p| extract_type_id(p)).flatten().collect::<Vec<_>>();
            deps.push(id.clone());
            deps
        }
        GenericType::Reference(typ) => extract_type_id(&typ),
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


