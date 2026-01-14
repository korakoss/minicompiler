use std::{collections::{BTreeMap, HashMap, VecDeque}};

type ConcreteCompositeType = CompositeType<ConcreteType>;
type GenericCompositeType = CompositeType<GenericType>; 

impl GenericCompositeType {
    pub fn monomorphize(&self, bindings: HashMap<String, ConcreteType>) -> ConcreteCompositeType {
        match self {
            Self::Struct { fields } => {
                ConcreteCompositeType::Struct { 
                    fields: fields.iter().map(|(fname, ftype)| (fname.clone(), ftype.monomorphize(bindings.clone()))).collect()
                }
            }
            Self::Enum { variants } => {
                ConcreteCompositeType::Enum { variants: variants.iter().map(|var| var.monomorphize(bindings.clone())).collect()}
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ConcreteType {
    Prim(PrimType),
    NewType(MonoTypeIdentifier),
    Reference(Box<ConcreteType>),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum GenericType {
    Prim(PrimType),
    NewType(PolyTypeIdentifier),
    Reference(Box<GenericType>),
    TypeVar(String)
}

impl GenericType {
    
    pub fn monomorphize(&self, bindings: HashMap<String, ConcreteType>) -> ConcreteType {
        match self {
            Self::Prim(prim_typ) => ConcreteType::Prim(*prim_typ),
            Self::NewType(PolyTypeIdentifier(id)) => ConcreteType::NewType(MonoTypeIdentifier { name: id.clone(), bindings }),
            Self::Reference(typ) => ConcreteType::Reference(Box::new(typ.monomorphize(bindings))),
            Self::TypeVar(id) => {
                bindings[id].clone()
            }
        }
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct GenericNewType {
    pub type_params: Vec<String>,
    pub defn: GenericCompositeType,
}


#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum CompositeType<T>{
    Struct {
        fields: BTreeMap<String, T>
    },
    Enum {
        variants: Vec<T>
    },
}


#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy)]
pub enum PrimType {
    Integer,
    Bool,
    None,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct PolyTypeIdentifier(pub String); 

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MonoTypeIdentifier {
    name: String,
    bindings: HashMap<String, ConcreteType>
}




// TODO: throw this out. TypeTable should only concern newtypes (and map to TypeConstructor). Logic related to handling other
// stuff should be handled elsewhere
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TypeDef {
    Prim(PrimType),
    NewType(CompositeType),
    Reference(Type)
}


// Maybe drop this too
// Or at least put elsewhere
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
    pub topo_order: Vec<PolyTypeIdentifier>,
    pub newtype_map: HashMap<PolyTypeIdentifier, CompositeType>,
}

impl TypeTable {

    pub fn make(newtype_defs: HashMap<PolyTypeIdentifier, CompositeType>) -> TypeTable {
        let topo_order = toposort_depgraph(get_newtype_dependencies(&newtype_defs));        
        TypeTable {
            topo_order,
            newtype_map: newtype_defs,
        }
    }

    pub fn get_typedef(&self, t: Type) -> TypeDef {
        match t {
            Type::Prim(prim_type) => TypeDef::Prim(prim_type),
            Type::NewType(id) => TypeDef::NewType(self.newtype_map[&id].clone()),
            Type::Reference(typ) => TypeDef::Reference(*typ),
        }
    }
}



fn get_newtype_dependencies(newtype_defs: &HashMap<PolyTypeIdentifier, CompositeType>) -> HashMap<PolyTypeIdentifier, Vec<PolyTypeIdentifier>> {
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


