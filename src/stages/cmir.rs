use std::collections::HashMap;

use crate::shared::ids::*;
use crate::shared::typing::*;
use crate::shared::{binops::BinaryOperator};

#[derive(Clone, Debug)]
pub struct CMIRProgram {
    pub functions: HashMap<FuncId, CMIRFunction>,
    pub entry: FuncId,
}

#[derive(Clone, Debug)]
pub struct CMIRFunction {
    pub name: String,
    pub args: Vec<CellId>,
    pub cells: HashMap<CellId, ConcreteType>,
    pub blocks: HashMap<BlockId, CMIRBlock>,
    pub entry: BlockId,
    pub ret_type: ConcreteType,
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




