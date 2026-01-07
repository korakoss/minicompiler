use crate::shared::binops::BinaryOperator;
use crate::shared::typing::*;
use std::{collections::HashMap};
use crate::hir::*;                  //for FuncID; TODO: put that somewhere shared
use crate::lir::*;                  // for blockID; likewise


struct MIRProgram {
    typetable: TypeTable,
    functions: HashMap<FuncId, MIRFunction>,
    entry: FuncId,
}

struct MIRFunction {
    name: String,
    args: Vec<VarId>,
    cells: HashMap<CellId, Cell>,
    blocks: HashMap<BlockId, MIRBlock>,
    entry: BlockId,
    ret_type: Type,
}

struct MIRBlock {
    statements: Vec<MIRStatement>,
    terminator: MIRTerminator,
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
enum MIRTerminator {
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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CellId(pub usize);
