use crate::shared::binops::*;
use crate::shared::utils::*;

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
    pub chunks: HashMap<ChunkId, Chunk>,
    pub args: Vec<ChunkId>
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
        args: Vec<LIRPlace>,
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
pub struct LIRValue {
    pub size: usize,
    pub value: LIRValueKind,
}


#[derive(Clone, Debug)]
pub enum LIRValueKind {
    Place(LIRPlace), 
    IntLiteral(i32),
    BoolTrue,
    BoolFalse,
    Reference(LIRPlace),
}

#[derive(Clone, Debug)]
pub struct LIRPlace {
    pub size: usize,
    pub place: LIRPlaceKind
}

#[derive(Clone, Debug)]
pub enum LIRPlaceKind {
    Local {
        base: ChunkId,
        offset: usize,
    },
    Deref {
        pointer: ChunkId,
        offset: usize,
    }
}

#[derive(Clone, Debug)]
pub struct Chunk {
    pub size: usize,
    // TODO: align or whatever
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy)]
pub struct ChunkId(pub usize);

#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy)]
pub struct VRegId(pub usize);
