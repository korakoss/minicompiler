use crate::shared::{binops::BinaryOperator, tables::GenericTypetable};
use crate::shared::typing::*;
use crate::shared::utils::*;

use std::{collections::HashMap};


#[derive(Clone, Debug)]
pub struct MIRProgram {
    pub typetable: GenericTypetable,
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
    pub ret_type: GenericType,
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
pub struct MIRValue {   
    pub typ: GenericType,
    pub value: MIRValueKind,
}

#[derive(Clone, Debug)]
pub enum MIRValueKind {
    Place(MIRPlace),        // Possible rename: stored
    IntLiteral(i32),
    BoolTrue,
    BoolFalse,
    StructLiteral {
        fields: HashMap<String, MIRValue>,
    },
    Reference(MIRPlace),
}


#[derive(Clone, Debug)]
pub struct MIRPlace {
    pub typ: GenericType,
    pub base: MIRPlaceBase,
    pub fieldchain: Vec<String>
}

#[derive(Clone, Debug)]
pub enum MIRPlaceBase {
    Cell(CellId),
    Deref(CellId),
}

#[derive(Clone, Debug)]
pub struct Cell {       // Could drop kind?
    pub typ: GenericType,
    pub kind: CellKind,
}

#[derive(Clone, Debug)]
pub enum CellKind {
    Var {
        name: String
    },
    Temp,
}


