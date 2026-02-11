use std::{collections::{BTreeMap, HashMap}, hash::Hash};

use crate::shared::typing::*;
use crate::shared::definitions::{IdFactory, NewtypeId, TypevarId};


#[derive(Debug, Clone)]
pub struct GenericTypetable {
    pub id_map: HashMap<String, NewtypeId>,
    pub defs: HashMap<NewtypeId, GenericTypeDef>,
    pub id_factory: IdFactory<NewtypeId>,
}

impl GenericTypetable {

    pub fn new() -> Self {
        Self { id_map: HashMap::new(), defs: HashMap::new(), id_factory: IdFactory::new() }
    }
    
    pub fn add_newtype(&mut self, name: String, def: GenericTypeDef) {
        let id = self.id_factory.next_id();
        self.id_map.insert(name, id);
        self.defs.insert(id, def);
    }

    pub fn get_type_id(&self, name: &String) -> Option<NewtypeId> {
        self.id_map.get(name).copied()
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
                        .map(|(name, typ)| (name, typ.monomorphize(&type_params).unwrap()))
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

