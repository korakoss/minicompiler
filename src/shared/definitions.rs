use std::hash::Hash; 
use crate::shared::typing::*;


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


#[derive(Debug, Clone)]
pub enum BinaryOperator {
    Add, 
    Sub, 
    Mul, 
    Equals,
    Less,       
    Modulo
}

pub fn binop_typecheck(op: &BinaryOperator, left_type: &GenericType, right_type: &GenericType) -> Option<GenericType> {
    
    match op {
        &BinaryOperator::Add | &BinaryOperator::Sub | &BinaryOperator::Mul| &BinaryOperator::Modulo=>{
            if left_type == &GenericType::Prim(PrimType::Integer) && right_type == &GenericType::Prim(PrimType::Integer){
                Some(GenericType::Prim(PrimType::Integer))
            } else {
                None
            }
        }
        &BinaryOperator::Equals => {
            if left_type == right_type {
                // TODO: careful later
                Some(GenericType::Prim(PrimType::Bool))
            } else {
                None
            }
        }
        &BinaryOperator::Less => {
            if left_type == &GenericType::Prim(PrimType::Integer) && right_type == &GenericType::Prim(PrimType::Integer){
                Some(GenericType::Prim(PrimType::Bool))
            } else {
                None
            }
        } 
    }
}


pub trait Id: Copy + Eq + Hash {
    fn from_raw(id: usize) -> Self;
    fn raw(self) -> usize;
}

macro_rules! define_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
        pub struct $name(usize);

        impl Id for $name {
            fn from_raw(id: usize) -> Self { Self(id) }
            fn raw(self) -> usize { self.0 }
        }
    };
}

define_id!(FuncId);
define_id!(BlockId);
define_id!(CellId);
define_id!(VarId);
define_id!(NewtypeId);
define_id!(TypevarId);


#[derive(Debug, Clone)]
pub struct IdFactory<I: Id> {
    counter: usize,
    _marker: std::marker::PhantomData<I>,
}

impl<I: Id> IdFactory<I> {

    pub fn new() -> Self {
        Self{ counter: 0, _marker: std::marker::PhantomData}
    }
    
    pub fn next_id(&mut self) -> I {
        let id = I::from_raw(self.counter);
        self.counter += 1;
        id
    }
}
