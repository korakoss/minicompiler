use crate::shared::binops::BinaryOperator;
use crate::shared::newtyping::*;
use crate::stages::common::*;

use std::{collections::HashMap};


#[derive(Clone, Debug)]
pub struct MIRProgram {
    pub typetable: TypeTable,
    pub functions: HashMap<FuncId, MIRFunction>,
    pub entry: FuncId,
}

#[derive(Clone, Debug)]
pub struct MIRFunction {
    pub name: String,
    pub args: Vec<CellId>,
    pub cells: HashMap<CellId, Cell>,
    pub blocks: HashMap<BlockId, MIRBlock>,
    pub entry: BlockId,
    pub ret_type: Type,
}

#[derive(Clone, Debug)]
pub struct MIRBlock {
    pub statements: Vec<MIRStatement>,
    pub terminator: MIRTerminator,
}

#[derive(Clone, Debug)]
pub enum MIRStatement {
    Assign {
        target: MIRPlace,
        value: MIRValue, 
    },
    BinOp {
        target: MIRPlace,
        op: BinaryOperator,
        left: MIRValue,
        right: MIRValue,
    },
    Call {
        target: MIRPlace,
        func: FuncId,
        args: Vec<MIRValue>,
    },
    Print(MIRValue),
}

#[derive(Clone, Debug)]
pub enum MIRTerminator {
    Goto(BlockId),
    Branch {
        condition: MIRValue,
        then_: BlockId,
        else_: BlockId,
    },
    Return(Option<MIRValue>),      
}

#[derive(Clone, Debug)]
pub enum MIRValue {
    Place(MIRPlace), 
    IntLiteral(i32),
    BoolTrue,
    BoolFalse,
    StructLiteral {
        typ: Type,
        fields: HashMap<String, MIRValue>,
    }
}

#[derive(Clone, Debug)]
pub struct MIRPlace {
    pub typ: Type,
    pub base: CellId,
    pub fieldchain: Vec<String>
}

#[derive(Clone, Debug)]
pub struct Cell {
    pub typ: Type,
    pub kind: CellKind,
}

#[derive(Clone, Debug)]
pub enum CellKind {
    Var {
        name: String
    },
    Temp,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy)]
pub struct CellId(pub usize);
