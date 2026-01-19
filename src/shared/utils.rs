use crate::shared::typing::*;

pub type GenericFuncSignature = FuncSignature<GenericType>;
pub type ConcreteFuncSignature = FuncSignature<ConcreteType>;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FuncSignature<T> {
    pub name: String,
    pub argtypes: Vec<T>,
}

pub type GenTypeVariable = Variable<GenericType>;
pub type ConcreteVariable = Variable<ConcreteType>;

// TODO: refactor this somehow
#[derive(Debug, Clone)]
pub struct Variable<T> {
    pub name: String,
    pub typ: T,
    // TODO: mutable, etc
}
