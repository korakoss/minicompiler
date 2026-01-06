use std::collections::{BTreeMap, HashMap, VecDeque};


pub type DeferredFunctionSignature = FuncSignature<DeferredType>;
pub type CompleteFunctionSignature = FuncSignature<Type>;

pub type DeferredTypeVariable = Variable<DeferredType>;
pub type TypedVariable = Variable<Type>;

pub type DeferredDerivType = TypeConstructor<DeferredType>;
pub type DerivType = TypeConstructor<Type>;


#[derive(Debug, Clone)]
pub struct Variable<T> {
    pub name: String,
    pub typ: T,
    // TODO: mutable, etc
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Type {
    Prim(PrimitiveType),
    Derived(TypeConstructor<Type>),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum DeferredType {
    Prim(PrimitiveType),
    Symbolic(TypeIdentifier),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PrimitiveType {
    Integer,
    Bool,
    None,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TypeConstructor<T>{
    Struct {
        fields: BTreeMap<String, T>
    },
    Enum {
        variants: Vec<T>
    }
}


#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FuncSignature<T> {
    pub name: String,
    pub argtypes: Vec<T>,
    // NOTE: maybe return type sometime?
}


#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct TypeIdentifier(pub String); 

#[derive(Clone, Debug)]
pub struct TypeTable {
    pub topo_order: Vec<TypeIdentifier>,
    pub newtype_map: HashMap<TypeIdentifier, DerivType>,
}

impl TypeTable {

    pub fn make(newtype_defs: HashMap<TypeIdentifier, DeferredDerivType>) -> TypeTable { 

        let dep_graph = get_newtype_dependencies(&newtype_defs); 
        let topo_order = toposort_depgraph(&dep_graph);

        let mut table = TypeTable{topo_order: topo_order.clone(), newtype_map: HashMap::new()}; 
        for type_id in topo_order { 
            let deferred_newtype = newtype_defs[&type_id].clone();
            let newtype = match deferred_newtype {
                TypeConstructor::Struct { fields } => TypeConstructor::Struct { 
                    fields: fields
                        .into_iter()
                        .map(|(nam, typ)| (nam, table.convert(typ)))
                        .collect()
                },         
                TypeConstructor::Enum { variants } => TypeConstructor::Enum { 
                    variants: variants 
                        .into_iter()
                        .map(|typ| table.convert(typ))
                        .collect()
                }
            };
            table.newtype_map.insert(type_id, newtype);
        }
        table 
    }

    pub fn convert(&self, t: DeferredType) -> Type {
        match t {
            DeferredType::Prim(prim) => Type::Prim(prim),
            DeferredType::Symbolic(type_id) => {
                let resolved = self.newtype_map
                    .get(&type_id)
                    .expect("Type symbol cannot be resolved").clone();
                Type::Derived(resolved)
            }
        }
    }
}


fn get_newtype_dependencies(newtype_defs: &HashMap<TypeIdentifier, DeferredDerivType>) -> HashMap<TypeIdentifier, Vec<TypeIdentifier>> {
    let mut dep_graph: HashMap<TypeIdentifier, Vec<TypeIdentifier>> = HashMap::new();
    for (type_id, newtype) in newtype_defs {
        let deps: Vec<TypeIdentifier> = match newtype {
            DeferredDerivType::Struct {fields} => fields
                .values()
                .filter_map(|ftyp| match ftyp {
                    DeferredType::Symbolic(id) => Some(id.clone()),
                    _ => None,
                })
                .collect(),
            DeferredDerivType::Enum { variants } => variants
                .iter()
                .filter_map(|vtyp| match vtyp {
                    DeferredType::Symbolic(id) => Some(id.clone()),
                    _ => None,
                })
                .collect()
        };
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
