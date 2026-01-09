use std::collections::{BTreeMap, HashMap, VecDeque};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Type {
    Prim(PrimType),
    NewType(TypeIdentifier),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TypeDef {
    Prim(PrimType),
    NewType(TypeConstructor)
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PrimType {
    Integer,
    Bool,
    None,
}


#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TypeConstructor{
    Struct {
        fields: BTreeMap<String, Type>
    },
    Enum {
        variants: Vec<Type>
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct TypeIdentifier(pub String); 


#[derive(Debug, Clone)]
pub struct Variable {
    pub name: String,
    pub typ: Type,
    // TODO: mutable, etc
}



#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FuncSignature {
    pub name: String,
    pub argtypes: Vec<Type>,
}

#[derive(Debug, Clone)]
pub struct TypeTable {
    pub topo_order: Vec<TypeIdentifier>,
    pub newtype_map: HashMap<TypeIdentifier, TypeConstructor>,
}

impl TypeTable {

    pub fn make(newtype_defs: HashMap<TypeIdentifier, TypeConstructor>) -> TypeTable {
        let topo_order = toposort_depgraph(get_newtype_dependencies(&newtype_defs));        
        TypeTable {
            topo_order,
            newtype_map: newtype_defs,
        }
    }

    pub fn get_typedef(&self, t: Type) -> TypeDef {
        match t {
            Type::Prim(prim_type) => TypeDef::Prim(prim_type),
            Type::NewType(id) => TypeDef::NewType(self.newtype_map[&id].clone())
        }
    }
}



fn get_newtype_dependencies(newtype_defs: &HashMap<TypeIdentifier, TypeConstructor>) -> HashMap<TypeIdentifier, Vec<TypeIdentifier>> {
    let mut dep_graph: HashMap<TypeIdentifier, Vec<TypeIdentifier>> = HashMap::new();
    for (type_id, newtype) in newtype_defs {
        let deps: Vec<TypeIdentifier> = match newtype {
            TypeConstructor::Struct {fields} => fields
                .values()
                .filter_map(|ftyp| match ftyp {
                    Type::NewType(id) => Some(id.clone()),
                    _ => None,
                })
                .collect(),
            TypeConstructor::Enum { variants } => variants
                .iter()
                .filter_map(|vtyp| match vtyp {
                    Type::NewType(id) => Some(id.clone()),
                    _ => None,
                })
                .collect()
        };
        dep_graph.insert(type_id.clone(), deps);
    }
    dep_graph
}

fn toposort_depgraph(depgraph: HashMap<TypeIdentifier, Vec<TypeIdentifier>>) -> Vec<TypeIdentifier> {

    let mut indegrees: HashMap<TypeIdentifier, usize> = depgraph
        .keys()
        .map(|k| (k.clone(),0))
        .collect();
    
   for neighbor in depgraph.values().flatten() {
        if let Some(count) = indegrees.get_mut(neighbor) {
            *count += 1;
        }
    }

    let mut queue: VecDeque<TypeIdentifier> = depgraph
        .keys()
        .filter(|node| indegrees[node] == 0)
        .cloned()
        .collect();
    let mut result: Vec<TypeIdentifier> = Vec::new();
    
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


