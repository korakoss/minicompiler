use crate::shared::typing::GenericType;
use crate::shared::ids::TypevarId;



pub type GenericFuncSignature = FuncSignature<GenericType>;

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

