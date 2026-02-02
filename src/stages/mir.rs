use std::{collections::HashMap};

use crate::shared::{
    binops::BinaryOperator, tables::GenericTypetable,
    typing::{GenericType, ConcreteType, TypevarId},
    ids::{BlockId, CellId, FuncId},
};


#[derive(Clone, Debug)]
pub struct MIRProgram {
    pub typetable: GenericTypetable,
    pub functions: HashMap<FuncId, MIRFunction>,
    pub entry: FuncId,
}

#[derive(Clone, Debug)]
pub struct MIRFunction {
    pub name: String,
    pub typvars: Vec<TypevarId>, 
    pub args: Vec<CellId>,
    pub cells: HashMap<CellId, GenericType>,
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
        type_params: Vec<GenericType>,
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
    Reference(MIRPlace),    // TODO: Clarify the semantics of this!
}


#[derive(Clone, Debug)]
pub struct MIRPlace {
    pub typ: GenericType,   // Type actually doesn't need to be stored here, maybe?
    pub base: MIRPlaceBase,
    pub fieldchain: Vec<String>
}

#[derive(Clone, Debug)]
pub enum MIRPlaceBase {
    Cell(CellId),
    Deref(CellId),
}

#[derive(Clone, Debug)]
pub struct CallGraph {
    calls: HashMap<FuncId, Vec<(FuncId, Vec<usize>)>>,    
    // Stores the callees along with a vector of type parameter indices to substitute 
}

impl CallGraph {
    
    pub fn new(ids: &Vec<FuncId>) -> Self {
        Self {
            calls: ids.iter().map(|i| (*i, vec![])).collect()
        }
    }
    
    pub fn get_concrete_callees(
        &self, 
        caller_id: &FuncId, 
        type_params: Vec<ConcreteType>
    ) -> Vec<(FuncId, Vec<ConcreteType>)> {
        self.calls[&caller_id]
            .iter()
            .map(|(id, param_indices)| (*id, param_indices
                .iter()
                .map(|idx| type_params[*idx].clone())
                .collect()
            ))
            .collect()
    }

    pub fn add_callee(&mut self, caller: &FuncId, callee: (FuncId, Vec<usize>)) {
        self.calls.get_mut(caller).unwrap().push(callee);
    }
}
