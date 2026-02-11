use std::hash::Hash; 
use anyhow::{Result, bail};

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

const INT_TYPE: GenericType = GenericType::Prim(PrimType::Integer);
const BOOL_TYPE: GenericType = GenericType::Prim(PrimType::Bool);

impl BinaryOperator {

    // TODO: change to Result
    pub fn type_result(&self, left: &GenericType, right: &GenericType) -> Result<GenericType> {
        match self {
            BinaryOperator::Add | BinaryOperator::Sub | BinaryOperator::Mul | BinaryOperator::Modulo => {
                if left == &INT_TYPE && right == &INT_TYPE {
                    return Ok(INT_TYPE);
                }
            },
            BinaryOperator::Equals => {
                if left == right {
                    return Ok(BOOL_TYPE);
                } 
            }
            BinaryOperator::Less => {
                if left == &INT_TYPE && right == &INT_TYPE {
                     return Ok(BOOL_TYPE);
                } 
            }
        }
        bail!("Binop typecheck failed");
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
