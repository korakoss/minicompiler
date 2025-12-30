use std::collections::BTreeMap;



pub type DeferredFunctionSignature = FuncSignature<DeferredType>;
pub type CompleteFunctionSignature = FuncSignature<Type>;

pub type DeferredTypeVariable = Variable<DeferredType>;
pub type TypedVariable = Variable<Type>;

pub type DeferredDerivType = TypeConstructor<DeferredType>;
pub type DerivType = TypeConstructor<Type>

#[derive(Debug, Clone)]
pub struct Variable<T> {
    pub name: String,
    pub typ: T,
    // TODO: mutable, etc
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Type {
    Prim(PrimitiveType),
    Derived(TypeConstructor<Type>),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum DeferredType {
    Prim(PrimitiveType),
    Symbolic(TypeIdentifier),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PrimitiveType {
    Integer,
    Bool,
    None,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TypeConstructor<T>{
    Struct {
        fields: BTreeMap<String, T>
    }
}


#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FuncSignature<T> {
    pub name: String,
    pub argtypes: Vec<T>,
    // NOTE: maybe return type sometime?
}


#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct TypeIdentifier(pub String); 









