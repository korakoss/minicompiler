use std::{collections::{BTreeMap}};


pub type GenericFuncSignature = FuncSignature<GenericType>;

pub type ConcreteCompositeType = CompositeType<ConcreteType>;
pub type GenericCompositeType = CompositeType<GenericType>; 

impl GenericCompositeType {

    pub fn monomorphize(&self, binding: &Binding) -> ConcreteCompositeType {
        match self {
            Self::Struct { fields } => {
                ConcreteCompositeType::Struct { 
                    fields: fields.iter().map(|(fname, ftype)| (fname.clone(), ftype.monomorphize(&binding))).collect()
                }
            }
            Self::Enum { variants } => {
                ConcreteCompositeType::Enum { variants: variants.iter().map(|var| var.monomorphize(&binding)).collect()}
            }
        }
    }
}

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
    
    pub fn monomorphize(&self, bindings: &Binding) -> ConcreteType {
        match self {
            Self::Prim(prim_typ) => ConcreteType::Prim(*prim_typ),
            Self::NewType(id, gen_params) => {
                let resolved_params = gen_params.iter().map(|p| p.monomorphize(bindings)).collect();
                ConcreteType::NewType(id.clone(), resolved_params)
                
            }
            Self::Reference(typ) => ConcreteType::Reference(Box::new(typ.monomorphize(bindings))),
            Self::TypeVar(id) => {
                bindings.resolve(&id)
            }
        }
    }
}


#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct GenericTypeDef {
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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Binding(BTreeMap<String, ConcreteType>);

impl Binding {
    pub fn resolve(&self, symbol: &String) -> ConcreteType {
        self.0.get(symbol).unwrap().clone()
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct NewtypeId(pub String); 


#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FuncSignature<T> {
    pub name: String,
    pub argtypes: Vec<T>,
}
