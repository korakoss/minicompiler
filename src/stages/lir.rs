use std::{collections::HashMap};

use crate::shared::definitions::{BinaryOperator, BlockId, FuncId, CellId};
use crate::passes::cmir_to_lir::ChunkLayout;

#[derive(Clone, Debug)]
pub struct LIRProgram {
    pub functions: HashMap<FuncId, LIRFunction>,
    pub entry: FuncId
}

#[derive(Clone, Debug)]
pub struct LIRFunction {
    pub blocks: HashMap<BlockId, LIRBlock>,
    pub entry: BlockId,
    pub chunks: StackFrame,     // TODO: rename to "frame"?
    pub args: Vec<CellId>
}


#[derive(Clone, Debug)]
pub struct StackFrame {
    chunk_sizes: Vec<(CellId, usize)>,
}

impl StackFrame {

    pub fn from_layouts(layouts: &HashMap<CellId, ChunkLayout>) -> Self {
        Self {
            chunk_sizes: layouts.iter().map(|(id, layout)| (*id, layout.size)).collect()
        }
    }
    
    pub fn size(&self) -> usize {
        self.chunk_sizes.iter().map(|(_, size)| size).sum()
    }
    
    pub fn get_offset(&self, chunk: &CellId) -> Option<usize> {
        let mut offset_acc = 8;
        for (id, size) in self.chunk_sizes.iter() {
            if chunk == id {
                return Some(offset_acc);
            }
            offset_acc += size;
        }
        None
    }
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
pub enum LIRValue {
    Place(LIRPlace), 
    IntLiteral(i32),
    BoolTrue,
    BoolFalse,
    Reference(LIRPlace),
}


#[derive(Clone, Debug)]
pub enum LIRPlace {
    Local {
        size: usize,        // TODO: actually make use of this rather than having the stack frame represent it
        base: CellId,
        offset: usize,
    },
    Deref {
        pointer: CellId,
        offset: usize,
    }
}

impl LIRPlace {
    pub fn increment_offset(&self, increment: usize) -> Self {
        match self {
            LIRPlace::Local { size, base, offset } => Self::Local {
                size: *size,
                base: *base,
                offset: offset + increment,
            },
            LIRPlace::Deref { pointer, offset } => Self::Deref { 
                pointer: *pointer, 
                offset: offset + increment 
            }
        }
    }
}

