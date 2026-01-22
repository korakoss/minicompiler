use std::collections::{BTreeMap};


#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ConcreteType {
    Prim(PrimType),
    NewType(NewtypeId, Vec<ConcreteType>),
    Reference(Box<ConcreteType>),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum GenericType {                      
    // This represent basically what we put in type annots and stuff, NOT the typedefs
    Prim(PrimType),
    NewType(NewtypeId, Vec<GenericType>),
    Reference(Box<GenericType>),
    TypeVar(String)
}

impl GenericType {

    pub fn bind(&self, bindings: &BTreeMap<String, GenericType>) -> GenericType {
        match self {
            Self::Prim(prim_typ) => GenericType::Prim(*prim_typ),
            Self::NewType(id, gen_params) => {
                let resolved_params = gen_params
                    .iter()
                    .map(|p| p.bind(bindings))
                    .collect();
                GenericType::NewType(id.clone(), resolved_params)
                
            }
            Self::Reference(typ) => {
                GenericType::Reference(Box::new(typ.bind(bindings)))
            }
            Self::TypeVar(id) => {
                bindings[id].clone()
            }
        }
    }
    
    pub fn monomorphize(&self, type_params: &BTreeMap<String, ConcreteType>) -> ConcreteType {
        match self {
            Self::Prim(prim_typ) => ConcreteType::Prim(*prim_typ),
            Self::NewType(id, gen_params) => {
                let resolved_params = gen_params
                    .iter()
                    .map(|p| p.monomorphize(type_params))
                    .collect();
                ConcreteType::NewType(id.clone(), resolved_params)
                
            }
            Self::Reference(typ) => {
                ConcreteType::Reference(Box::new(typ.monomorphize(type_params)))
            }
            Self::TypeVar(id) => {
                type_params[id].clone()
            }
        }
    }
}


pub type GenericTypeDef = NewtypeDef<GenericType>;
pub type ConcreteTypeDef = NewtypeDef<ConcreteType>;


#[derive(Clone, Debug)]
pub struct NewtypeDef<T> {
    pub type_params: Vec<String>,
    pub defn: NewtypeShape<T>,
}

pub type GenericShape = NewtypeShape<GenericType>;
pub type ConcreteShape = NewtypeShape<ConcreteType>;


#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum NewtypeShape<T>{
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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Binding(pub BTreeMap<String, ConcreteType>);

impl Binding {
    pub fn resolve(&self, symbol: &String) -> ConcreteType {
        self.0.get(symbol).unwrap().clone()
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct NewtypeId(pub String); 



