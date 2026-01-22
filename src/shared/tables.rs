use std::{collections::{BTreeMap, HashMap, VecDeque}, hash::Hash};
use crate::shared::typing::*;



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
            monomorphizations: defs.iter().map(|(id, _)| (id.clone(), HashMap::new())).collect(),
            defs,
        }
    }

    pub fn get_mono(&self, id: NewtypeId, tvars: Vec<ConcreteType>) -> ConcreteShape {
        self.monomorphizations[&id][&tvars].clone()
    }

    pub fn topo_mono_iter(&self) -> impl Iterator<Item = (NewtypeId, Vec<ConcreteType>, ConcreteShape)> {
        let mut monomorphizations = self.monomorphizations.clone();
        self.topo_order
            .iter()
            .flat_map(move |id| {
                monomorphizations.remove(&id)
                    .unwrap_or_default()
                    .into_iter()
                    .map(move |(tvars, shape)| (id.clone(), tvars, shape))
            })
    }


    pub fn monomorphize(
        &mut self, 
        id: NewtypeId, 
        typ_var_vals: Vec<ConcreteType>
    ) -> ConcreteShape {
        let def = self.defs[&id].clone();
        let bindings: BTreeMap<String, ConcreteType> = def.type_params
            .iter()
            .cloned()
            .zip(typ_var_vals.iter().cloned())
            .collect();
        let monomorph = match def.defn {
            NewtypeShape::Struct { fields } => {
                NewtypeShape::Struct { 
                    fields: fields
                        .into_iter()
                        .map(|(name, typ)| (name, typ.monomorphize(&bindings)))
                        .collect()
                }
            }
            NewtypeShape::Enum { variants } => unimplemented!()
        };
        self.monomorphizations.get_mut(&id).unwrap().insert(typ_var_vals,monomorph.clone());
        monomorph
    }
}





fn extract_newtype_dependencies(newtype_defs: &HashMap<NewtypeId, GenericTypeDef>) -> HashMap<NewtypeId, Vec<NewtypeId>> {
    let mut dep_graph: HashMap<NewtypeId, Vec<NewtypeId>> = HashMap::new();
    for (type_id, newtype) in newtype_defs {
        let deps: Vec<NewtypeId> = match &newtype.defn {
            NewtypeShape::Struct {fields} => fields
                .values()
                .map(|ftyp| extract_type_id(&ftyp))
                .flatten()
                .collect(),
            NewtypeShape::Enum { variants } => variants
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
