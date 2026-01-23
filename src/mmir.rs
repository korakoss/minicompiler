

// TODO: rename to CMIR (for Concrete) if kept, and rename the other to GMIR

use crate::shared::{binops::BinaryOperator};
use crate::shared::typing::*;
use crate::stages::common::*;

use std::{collections::HashMap};

#[derive(Clone, Debug)]
pub struct ConcreteTypetable {
    concrete_newtypes: HashMap<ConcreteType, ConcreteShape>,
    topo_order: Vec<ConcreteType>
}


#[derive(Clone, Debug)]
pub struct CMIRProgram {
    pub typetable: ConcreteTypetable,
    pub functions: HashMap<FuncId, CMIRFunction>,
    pub entry: FuncId,
}

#[derive(Clone, Debug)]
pub struct CMIRFunction {
    pub name: String,
    pub args: Vec<CellId>,
    pub cells: HashMap<CellId, ConcreteCell>,
    pub blocks: HashMap<BlockId, CMIRBlock>,
    pub entry: BlockId,
    pub ret_type: GenericType,
}

#[derive(Clone, Debug)]
pub struct CMIRBlock {
    pub statements: Vec<CMIRStatement>,
    pub terminator: CMIRTerminator,
}

#[derive(Clone, Debug)]
pub enum CMIRStatement {
    Assign {
        target: CMIRPlace,
        value: CMIRValue, 
    },
    BinOp {
        target: CMIRPlace,
        op: BinaryOperator,
        left: CMIRValue,
        right: CMIRValue,
    },
    Call {
        target: CMIRPlace,
        func: FuncId,
        args: Vec<CMIRValue>,
    },
    Print(CMIRValue),
}

#[derive(Clone, Debug)]
pub enum CMIRTerminator {
    Goto(BlockId),
    Branch {
        condition: CMIRValue,
        then_: BlockId,
        else_: BlockId,
    },
    Return(Option<CMIRValue>),      
}

#[derive(Clone, Debug)]
pub struct CMIRValue {
    pub typ: ConcreteType,
    pub value: CMIRValueKind,
}

#[derive(Clone, Debug)]
pub enum CMIRValueKind {
    Place(CMIRPlace), 
    IntLiteral(i32),
    BoolTrue,
    BoolFalse,
    StructLiteral {
        typ: ConcreteType,                  // TODO refactor: storing type in V and VKind is
        // redundant
        fields: HashMap<String, CMIRValue>,
    },
    Reference(CMIRPlace),
}


#[derive(Clone, Debug)]
pub struct CMIRPlace {
    pub typ: ConcreteType,
    pub base: CMIRPlaceBase,
    pub fieldchain: Vec<String>
}

#[derive(Clone, Debug)]
pub enum CMIRPlaceBase {
    Cell(CellId),
    Deref(CellId),
}

#[derive(Clone, Debug)]
pub struct ConcreteCell {
    pub typ: ConcreteType,
    pub kind: CellKind,
}

#[derive(Clone, Debug)]
pub enum CellKind {
    Var {
        name: String
    },
    Temp,
}


