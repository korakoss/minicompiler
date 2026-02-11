use std::collections::{BTreeMap};
use anyhow::{Result, anyhow};

use crate::shared::definitions::{TypevarId, NewtypeId};


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
    TypeVar(TypevarId),
}

impl GenericType {

    pub fn bind(&self, bindings: &BTreeMap<TypevarId, GenericType>) -> GenericType {
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
    
    pub fn monomorphize(&self, type_params: &BTreeMap<TypevarId, ConcreteType>) -> Result<ConcreteType> {
        // TODO: check lengths, handle mismatch
        match self {
            Self::Prim(prim_typ) => Ok(ConcreteType::Prim(*prim_typ)),
            Self::NewType(id, gen_params) => {
                let resolved_params: Result<Vec<_>> = gen_params
                    .iter()
                    .map(|p| p.monomorphize(type_params))
                    .collect();
                Ok(ConcreteType::NewType(id.clone(), resolved_params?))
            }
            Self::Reference(typ) => {
                Ok(ConcreteType::Reference(Box::new(typ.monomorphize(type_params)?)))
            }
            Self::TypeVar(id) => {
                Ok(type_params
                    .get(id)
                    .ok_or_else(|| anyhow!("Type variable {:?} not found among type params {:?}", id, type_params))?
                    .clone())
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
