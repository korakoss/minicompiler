use crate::shared::typing::*;

pub type GenericFuncSignature = FuncSignature<GenericType>;
pub type ConcreteFuncSignature = FuncSignature<ConcreteType>;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FuncSignature<T> {
    pub name: String,
    pub argtypes: Vec<T>,
}

// TODO: Probably Concrete is not needed actually, since we switchh out to L/R-vals by then
pub type GenTypeVariable = Variable<GenericType>;
pub type ConcreteVariable = Variable<ConcreteType>;

// TODO: refactor this somehow
#[derive(Debug, Clone)]
pub struct Variable<T> {
    pub name: String,
    pub typ: T,
    // TODO: mutable, etc
}


#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct FuncId(pub usize);

#[derive(Clone, Debug, Eq, PartialEq, Hash, Copy)]
pub struct BlockId(pub usize);

#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy)]
pub struct CellId(pub usize);
