use std::collections::BTreeMap;



pub type DeferredFunctionSignature = FuncSignature<DeferredType>;
pub type CompleteFunctionSignature = FuncSignature<Type>;

pub type DeferredTypeVariable = Variable<DeferredType>;
pub type TypedVariable = Variable<Type>;

pub type DeferredNewType = NewType<DeferredType>;
pub type CompleteNewType = NewType<Type>;

#[derive(Debug, Clone)]
pub struct Variable<T> {
    pub name: String,
    pub typ: T,
    // TODO: mutable, etc
}


#[derive(PartialEq,Eq, Debug, Hash, Clone)]
pub enum Type {
    Primitive(PrimitiveType), 
    NewType(NewType<Type>),
}

#[derive(PartialEq,Eq, Debug, Hash, Clone)]
pub enum PrimitiveType {
    Integer,
    Bool,
    None,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DeferredType {
    Resolved(Type),
    Unresolved(TypeIdentifier)
}

#[derive(PartialEq,Eq, Debug, Hash, Clone)]
pub enum NewType<T> {
    Struct {
        fields: BTreeMap<String, T>,
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









