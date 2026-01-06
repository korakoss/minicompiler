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
    /*
    Enum {
        variants: Vec<T>
    }
    */
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
            let TypeConstructor::Struct { fields } = deferred_newtype.clone();
            table.add_struct_def(type_id, fields);
        }
        table 
    }

    fn add_struct_def(&mut self, type_id: TypeIdentifier, struct_fields: BTreeMap<String, DeferredType>) {
        let mut tfields : BTreeMap<String, Type> = BTreeMap::new();
        for (fname, ftype) in struct_fields {
            let actual_type = match ftype {
                DeferredType::Prim(prim_typ) => Type::Prim(prim_typ),
                DeferredType::Symbolic(type_id) => Type::Derived(self.newtype_map[&type_id].clone()),
            };
            tfields.insert(fname, actual_type);
        }
        let complete_newtype = TypeConstructor::Struct { fields: tfields};
        self.newtype_map.insert(type_id.clone(), complete_newtype.clone());
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
