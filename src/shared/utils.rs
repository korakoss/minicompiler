use crate::shared::typing::*;

pub type GenericFuncSignature = FuncSignature<GenericType>;
pub type ConcreteFuncSignature = FuncSignature<ConcreteType>;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FuncSignature<T> {
    pub name: String,
    pub typevars: Vec<TypevarId>,
    pub argtypes: Vec<T>,
}

#[derive(Clone, Debug)]
pub struct GenTypeVariable {
    pub name: String,
    pub typ: GenericType,
}

