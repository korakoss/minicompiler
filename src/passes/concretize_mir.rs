use std::collections::HashMap;

use crate::shared::utils::FuncId;
use crate::stages::{lir::*, mir::*, cmir::*};
use crate::shared::{
    typing::{GenericType, ConcreteType, PrimType},
    tables::{GenericTypetable, ConcreteShape},
    utils::{CellId},
};


pub struct MIRLowerer {
    pub monomorphized_functions: HashMap<(FuncId, Vec<ConcreteType>), CMIRFunction>,
    pub generic_functions: HashMap<FuncId, MIRFunction>,
    pub typetable: GenericTypetable,
    pub entry: FuncId
}

impl  MIRLowerer {
    
    pub fn lower_mir(program: MIRProgram) -> CMIRProgram {
        unimplemented!();
    }

    pub fn monomomorphize_function(
        &mut self, 
        func: MIRFunction, 
        type_params: Vec<ConcreteType>
    ) -> CMIRFunction {
        // Monomorphize types of cells
        // Monomorphize return type
        // Iterate through the blocks of the function body
        // Within the blocks, the statements
        // Monomorphize the Place/Value types too
        CMIRFunction {
            name: func.name,
            args: unimplemented!(),
            cells: unimplemented!(),
            blocks: unimplemented!(),
            entry: unimplemented!(),
            ret_type: unimplemented!(),
        }
    }

}

