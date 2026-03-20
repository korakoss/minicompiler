use std::{collections::HashMap};

use crate::shared::{
    definitions::{BinaryOperator, BlockId, CellId, FuncId, VregId},
    tables::ChunkLayout,
};

#[derive(Clone, Debug)]
pub struct LIRProgram {
    pub functions: HashMap<FuncId, LIRFunction>,
    pub entry: FuncId,
}

#[derive(Clone, Debug)]
pub struct LIRFunction {
    pub blocks: HashMap<BlockId, LIRBlock>,
    pub entry: BlockId,
    pub frame: StackFrame, 
    pub args: Vec<VregId>,
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

#[derive(Clone, Copy, Debug)]
pub struct LIRPlace {
    pub base: Address,
    pub offset: usize,
}

impl LIRPlace {
    pub fn increase_offset(&self, increment: usize) -> Self {
        Self {
            base: self.base,
            offset: self.offset + increment,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Address {
    Chunk(MemoryChunk),
    Dereference(MemoryChunk),
}

#[derive(Clone, Copy, Debug)]
pub enum MemoryChunk {
    Local(CellId),
    VReg(VregId),
}


#[derive(Clone, Debug, Copy)]
pub struct Cell {
    id: CellId,
    size: usize,
}

#[derive(Clone, Debug)]
pub struct StackFrame {
    cells: Vec<Cell>,
}

impl StackFrame {

    pub fn from_layouts(layouts: &HashMap<CellId, ChunkLayout>) -> Self {
        Self {
            cells: layouts.iter()
                .map(|(id, layout)| Cell { id: *id, size: layout.size})
                .collect()
        }
    }
    
    pub fn size(&self) -> usize {
        self.cells.iter().map(|cell| cell.size).sum()
    }
    
    pub fn get_offset(&self, chunk: &CellId) -> Option<usize> {
        let mut offset_acc = 8;
        for cell in self.cells.iter() {
            if cell.id == *chunk {
                return Some(offset_acc);
            }
            offset_acc += cell.size;
        }
        None
    }
}

