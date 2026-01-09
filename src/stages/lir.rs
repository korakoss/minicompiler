use crate::shared::binops::*;
use crate::stages::common::*;
use crate::shared::typing::*;

use std::{collections::HashMap};


#[derive(Clone, Debug)]
pub struct LIRProgram {
    pub functions: HashMap<FuncId, LIRFunction>,
    pub entry: FuncId
}

#[derive(Clone, Debug)]
pub struct LIRFunction {
    pub blocks: HashMap<BlockId, LIRBlock>,
    pub entry: BlockId,
    pub vregs: HashMap<VRegId, VRegInfo>,
    pub args: Vec<VRegId>
}

#[derive(Clone, Debug)]
pub struct LIRBlock {
    pub statements: Vec<LIRStatement>,
    pub terminator: LIRTerminator,
}


#[derive(Clone, Debug)]
pub enum LIRStatement {
    Store {
        dest: LIRPlace,
        value: LIRValue,
    },

    BinOp {
        dest: LIRPlace,
        op: BinaryOperator,
        left: LIRValue,
        right: LIRValue,
    },
    Call {
        dest: LIRPlace,
        func: FuncId,
        args: Vec<LIRValue>,
    },
    Print(LIRValue),
}


#[derive(Clone, Debug)]
pub enum LIRTerminator {
    Goto {
        dest: BlockId,
    },
    Branch {
        condition: LIRValue,
        then_block: BlockId,
        else_block: BlockId
    },
    Return(Option<LIRValue>)
}


#[derive(Clone, Debug)]
pub enum LIRValue {
    Place{
        typ: Type,
        place: LIRPlace
    }, 
    IntLiteral(i32),
    BoolTrue,
    BoolFalse,
}

#[derive(Clone, Debug)]
pub struct LIRPlace {
    pub base: VRegId,
    pub offset: usize,
}

#[derive(Clone, Debug)]
pub struct VRegInfo{
    pub size: usize,
    pub align: usize,
}


#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct VRegId(pub usize);
