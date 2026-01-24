use std::collections::{BTreeMap};


#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ConcreteType {
    Prim(PrimType),
    NewType(NewtypeId, Vec<ConcreteType>),  // Should we separate it out in its own (gen) struct?
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

    // NOTE: this method is an utility for development purposes
    pub fn coerce_concrete(&self) -> ConcreteType {
        match self {
            Self::Prim(prim_typ) => ConcreteType::Prim(*prim_typ),
            Self::NewType(id, type_params) => {
                ConcreteType::NewType(id.clone(), type_params
                    .iter()
                    .map(|p| p.coerce_concrete())
                    .collect()
                )
            },
            Self::Reference(typ) => {
                ConcreteType::Reference(Box::new(typ.coerce_concrete()))
            },
            Self::TypeVar(..) => {panic!("Typevars cannot be coerced");}
        }
    }

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


#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy)]
pub enum PrimType {
    Integer,
    Bool,
    None,
}


#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct NewtypeId(pub String); 
