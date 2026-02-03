use std::{collections::{BTreeMap, HashMap}, hash::Hash};
use crate::shared::typing::*;


#[cfg(test)]
#[path = "../tests/test_tables.rs"]
mod tests;


#[derive(Debug, Clone)]
pub struct GenericTypetable {
    pub defs: HashMap<NewtypeId, GenericTypeDef>,
}

impl GenericTypetable {

    pub fn new(defs: HashMap<NewtypeId, GenericTypeDef>) -> Self {
        Self { defs}
    }
    
    pub fn bind(
        &self, 
        id: NewtypeId, 
        typ_var_vals: Vec<GenericType>
    ) -> GenericShape {
        let def = self.defs[&id].clone();
        let type_params: BTreeMap<TypevarId, GenericType> = def.type_params
            .iter()
            .cloned()
            .zip(typ_var_vals.iter().cloned())
            .collect();
        match def.defn {
            GenericShape::Struct { fields } => {
                GenericShape::Struct { 
                    fields: fields
                        .into_iter()
                        .map(|(name, typ)| (name, typ.bind(&type_params)))
                        .collect()
                }
            }
            GenericShape::Enum {..} => {
                unimplemented!();
            }
        }
    }

    pub fn monomorphize(
        &self, 
        id: NewtypeId, 
        typ_var_vals: Vec<ConcreteType>
    ) -> ConcreteShape {
        let def = self.defs[&id].clone();
        let type_params: BTreeMap<TypevarId, ConcreteType> = def.type_params
            .iter()
            .cloned()
            .zip(typ_var_vals.iter().cloned())
            .collect();
        match def.defn {
            GenericShape::Struct { fields } => {
                ConcreteShape::Struct { 
                    fields: fields
                        .into_iter()
                        .map(|(name, typ)| (name, typ.monomorphize(&type_params)))
                        .collect()
                }
            }
            GenericShape::Enum {..} => {
                unimplemented!();
            }
        }
    }

    pub fn get_genericity_rank(
        &self,
        typ: &ConcreteType,
    ) -> usize {
        match typ {
            ConcreteType::Prim(..) => 0,
            ConcreteType::Reference(ref_typ) => self.get_genericity_rank(ref_typ) + 1,
            ConcreteType::NewType(id, type_params) => {
                let type_shape = self.monomorphize(id.clone(), type_params.clone());
                match type_shape {
                    ConcreteShape::Struct { fields } => {
                        fields
                            .values()
                            .map(|typ| self.get_genericity_rank(typ))
                            .max()
                            .unwrap() + 1
                    },
                    ConcreteShape::Enum { .. } => {
                        unimplemented!();
                    }
                }
            }
        }
    }
}


pub type GenericTypeDef = NewtypeDef<GenericType>;
pub type GenericShape = NewtypeShape<GenericType>;

pub type ConcreteShape = NewtypeShape<ConcreteType>;
pub type ConcreteTypeDef = NewtypeDef<ConcreteType>;


#[derive(Clone, Debug)]
pub struct NewtypeDef<T> {
    pub type_params: Vec<TypevarId>,
    pub defn: NewtypeShape<T>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum NewtypeShape<T>{
    Struct {
        fields: BTreeMap<String, T>
    },
    Enum {
        variants: Vec<T>
    },
}

