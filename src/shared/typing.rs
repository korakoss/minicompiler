use std::{collections::{BTreeMap}};

pub type ConcreteCompositeType = CompositeType<ConcreteType>;
pub type GenericCompositeType = CompositeType<GenericType>; 

impl GenericCompositeType {
    pub fn monomorphize(&self, binding: &Binding) -> ConcreteCompositeType {
        match self {
            Self::Struct { fields } => {
                ConcreteCompositeType::Struct { 
                    fields: fields.iter().map(|(fname, ftype)| (fname.clone(), ftype.monomorphize(binding.clone()))).collect()
                }
            }
            Self::Enum { variants } => {
                ConcreteCompositeType::Enum { variants: variants.iter().map(|var| var.monomorphize(binding.clone())).collect()}
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
    
    pub fn monomorphize(&self, bindings: Binding) -> ConcreteType {
        match self {
            Self::Prim(prim_typ) => ConcreteType::Prim(*prim_typ),
            Self::NewType(PolyTypeIdentifier(id)) => ConcreteType::NewType(MonoTypeIdentifier { name: id.clone(), bindings }),
            Self::Reference(typ) => ConcreteType::Reference(Box::new(typ.monomorphize(bindings))),
            Self::TypeVar(id) => {
                bindings.resolve(&id)
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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Binding(BTreeMap<String, ConcreteType>);

impl Binding {
    pub fn resolve(&self, symbol: &String) -> ConcreteType {
        self.0.get(symbol).unwrap().clone()
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct PolyTypeIdentifier(pub String); 

impl PolyTypeIdentifier {
    
    pub fn bind(&self, binding: &Binding) -> MonoTypeIdentifier {
        let PolyTypeIdentifier(name) = self;
        MonoTypeIdentifier { 
            name: name.clone(), 
            bindings: binding.clone()
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MonoTypeIdentifier {
    name: String,
    bindings: Binding,
}





// Trashcan (probably -- or at least move)


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


