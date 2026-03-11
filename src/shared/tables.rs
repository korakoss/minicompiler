use std::{collections::{BTreeMap, HashMap, HashSet}, hash::Hash};

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
                let type_shape = self.monomorphize(*id, type_params.clone());
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



pub struct LayoutTable {
    pub layouts: HashMap<(NewtypeId, Vec<ConcreteType>), ChunkLayout>,
}

impl LayoutTable {

    pub fn make(typetable: GenericTypetable ,concrete_newtypes: HashSet<(NewtypeId, Vec<ConcreteType>)>) -> Self {
        let mut table = Self { layouts: HashMap::new() };
        let mut concrete_newtypes = concrete_newtypes
            .into_iter()
            .collect::<Vec<_>>();
        concrete_newtypes.sort_by_key(|(id, typ)| typetable.get_genericity_rank(&ConcreteType::NewType(*id, typ.clone())));
        for (id, tparams) in concrete_newtypes {
            let type_shape = typetable.monomorphize(id, tparams.clone());
            let type_layout = match type_shape {
                ConcreteShape::Struct { fields } => {
                    let fields: Vec<(String, ConcreteType)> = fields.into_iter().collect();
                    ChunkLayout {
                        size: fields.iter().map(|(_ ,ftyp)| table.get_layout(ftyp).size).sum(),
                        typ: ConcreteType::NewType(id, tparams.clone()),
                        kind: LayoutKind::Struct(fields),
                    }
                },
                ConcreteShape::Enum {..} => {
                    unimplemented!();
                },
            };
            table.layouts.insert((id, tparams), type_layout);
        }
        table
    }

    pub fn get_layout(&self, typ: &ConcreteType) -> ChunkLayout {
        match typ {
            ConcreteType::Prim(..) => ChunkLayout { size: 8, typ: typ.clone(), kind: LayoutKind::Atomic },
            ConcreteType::Reference(..) => ChunkLayout { size: 8, typ: typ.clone(), kind: LayoutKind::Atomic },
            ConcreteType::NewType(id, tparams) => self.layouts[&(*id, tparams.clone())].clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ChunkLayout {
    pub size: usize,                        // NOTE: maybe align etc here too
    pub typ: ConcreteType,
    pub kind: LayoutKind,
}

#[derive(Clone, Debug)]
pub enum LayoutKind {
    Atomic,                                 // primitives, pointers -- no internal structure basically
    Struct(Vec<(String, ConcreteType)>),    // it's been given a fixed order
}
