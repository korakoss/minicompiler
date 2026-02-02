use std::collections::{BTreeMap, HashMap};

use crate::shared::typing::{NewtypeId, TypevarId};
use crate::stages::{mir::*, cmir::*};
use crate::shared::{
    typing::{ConcreteType},
    tables::{GenericTypetable},
    ids::{BlockId, CellId, FuncId, IdFactory}
};


pub struct MIRLowerer {
    mono_funcs_map: HashMap<(FuncId, Vec<ConcreteType>), FuncId>, 
    current_mono_stack: Vec<(FuncId, Vec<ConcreteType>)>,
    mono_requests: Vec<(FuncId, Vec<ConcreteType>)>,
    generic_functions: HashMap<FuncId, MIRFunction>,
    mono_types: Vec<(NewtypeId, Vec<ConcreteType>)>,
    typetable: GenericTypetable,
    blockid_factory: IdFactory<BlockId>,
    cellid_factory: IdFactory<CellId>,
    funcid_factory: IdFactory<FuncId>,
 }

impl  MIRLowerer {
    
    pub fn lower_mir(program: MIRProgram) -> CMIRProgram {
        let mut lowerer = MIRLowerer {
            mono_funcs_map: HashMap::new(),
            current_mono_stack: Vec::new(),
            mono_requests: Vec::new(),
            generic_functions: program.functions,
            mono_types: Vec::new(),
            typetable: program.typetable,
            blockid_factory: IdFactory::new(),
            cellid_factory: IdFactory::new(),
            funcid_factory: IdFactory::new(),
        };
        let low_functions = lowerer.lower_functions(program.entry);
        CMIRProgram { 
            functions: low_functions, 
            entry: lowerer.mono_funcs_map[&(program.entry, vec![])] 
        }
    }

    pub fn lower_functions(&mut self, entry: FuncId) -> HashMap<FuncId, CMIRFunction>{
        // TODO: finish this
        let _entry_id = self.blockid_factory.next_id();
        self.mono_requests.push((entry, vec![])); 
        let low_funcs: HashMap<FuncId, CMIRFunction> = HashMap::new();
        while let Some((id, type_params)) = self.mono_requests.pop() {
            self.monomomorphize_function(id, type_params);
        }
        low_funcs
    }

    pub fn monomomorphize_function(
        &mut self, 
        id: FuncId, 
        type_params: Vec<ConcreteType>
    ) -> CMIRFunction {
        let gen_func= self.generic_functions[&id].clone(); 
        let tparam_bindings = gen_func.typvars
            .iter()
            .cloned()
            .zip(type_params.iter().cloned())
            .collect();
        
        let mut lowered_cells: HashMap<CellId, ConcreteType> = HashMap::new();
        let mut cell_map: HashMap<CellId, CellId> = HashMap::new();

        for (id, typ) in gen_func.cells {
            let low_id = self.cellid_factory.next_id();
            lowered_cells.insert(low_id, typ.monomorphize(&tparam_bindings));
            cell_map.insert(id, low_id);
        }

        let mut block_map: HashMap<BlockId, BlockId> = HashMap::new();
        
        for id in gen_func.blocks.keys() {
            let low_id = self.blockid_factory.next_id();
            block_map.insert(*id, low_id);
        }
        unimplemented!();
        

        // Pre-generate the lowered block IDs, then lower the blocks
        // Map in the args, entries, etc
        // Map ret-type
        /*
        CMIRFunction {
            name: gen_func.name,
            args: gen_func.args.into_iter().map(|arg_cell| cell_map[&arg_cell]).collect(),
            cells: lowered_cells,
            blocks: unimplemented!(),
            entry: unimplemented!(),
            ret_type: gen_func.ret_type.monomorphize(&tparam_bindings),
        }
        */
    }

    fn lower_block(
        &mut self,
        block: MIRBlock, 
        bindings: &BTreeMap<TypevarId, ConcreteType>,
        block_map: &HashMap<BlockId, BlockId>,
        cell_map: &HashMap<CellId, CellId>,
    ) -> CMIRBlock {
        let mut low_statements: Vec<CMIRStatement> = Vec::new();
        for stmt in block.statements {
            let low_stmt = match stmt {
                MIRStatement::Assign { target, value } => {
                    CMIRStatement::Assign { 
                        target: self.lower_place(target, bindings, cell_map), 
                        value: self.lower_value(value, bindings, cell_map),
                    } 
                },
                MIRStatement::BinOp { target, op, left, right } => {
                    CMIRStatement::BinOp { 
                        target: self.lower_place(target, bindings, cell_map),
                        op, 
                        left: self.lower_value(left, bindings, cell_map), 
                        right: self.lower_value(right, bindings, cell_map)
                    }
                },
                MIRStatement::Call { target, func, type_params, args } => {
                    let concrete_type_params = type_params
                        .into_iter()
                        .map(|typ| typ.monomorphize(bindings))
                        .collect();
                    // TODO: do this right
                    let low_func = self.mono_funcs_map[&(func, concrete_type_params)];
                    CMIRStatement::Call { 
                        target: self.lower_place(target, bindings, cell_map), 
                        func: low_func, 
                        args: args
                            .into_iter()
                            .map(|val| self.lower_value(val, bindings, cell_map))
                            .collect(),
                    }
                },
                MIRStatement::Print(value) => {
                    CMIRStatement::Print(self.lower_value(value, bindings, cell_map))
                },
            };
            low_statements.push(low_stmt);
        }
        CMIRBlock {
            statements: low_statements,
            terminator: self.lower_terminator(block.terminator, block_map, bindings, cell_map),
        }
    }

    fn lower_terminator(
        &mut self, 
        term:MIRTerminator, 
        block_map: &HashMap<BlockId, BlockId>,
        bindings: &BTreeMap<TypevarId, ConcreteType>,
        cell_map: &HashMap<CellId, CellId>
    ) -> CMIRTerminator {
        match term {
            MIRTerminator::Goto(id) => CMIRTerminator::Goto(block_map[&id]),
            MIRTerminator::Branch { condition, then_, else_ } => {
                CMIRTerminator::Branch { 
                    condition: self.lower_value(condition, bindings, cell_map),
                    then_: block_map[&then_],
                    else_: block_map[&else_],
                }
            }
            MIRTerminator::Return(val) => {
                CMIRTerminator::Return(val.map(|some_val| self.lower_value(some_val, bindings, cell_map)))
            }
        }
    }

    fn lower_value(
        &self, 
        val: MIRValue, 
        bindings: &BTreeMap<TypevarId, ConcreteType>,
        cell_map: &HashMap<CellId, CellId>
    ) -> CMIRValue {
        let low_value = match val.value {
            MIRValueKind::Place(place) => {
                CMIRValueKind::Place(self.lower_place(place, bindings, cell_map))
            }
            MIRValueKind::IntLiteral(num) => CMIRValueKind::IntLiteral(num),
            MIRValueKind::BoolTrue => CMIRValueKind::BoolTrue,
            MIRValueKind::BoolFalse => CMIRValueKind::BoolFalse,
            MIRValueKind::StructLiteral { fields } => {
                CMIRValueKind::StructLiteral { 
                    fields: fields
                        .into_iter()
                        .map(|(name, val)| (name, self.lower_value(val, bindings, cell_map)))
                        .collect()
                }
            }
            MIRValueKind::Reference(place) => {
                CMIRValueKind::Reference(self.lower_place(place, bindings, cell_map))
            }
        };
        CMIRValue { 
            typ: val.typ.monomorphize(bindings), 
            value: low_value 
        }
    }

    fn lower_place(
        &self, 
        place: MIRPlace, 
        bindings: &BTreeMap<TypevarId, ConcreteType>,
        cell_map: &HashMap<CellId, CellId>
    ) -> CMIRPlace {
        let low_base = match place.base {
            MIRPlaceBase::Cell(id) => CMIRPlaceBase::Cell(cell_map[&id]),
            MIRPlaceBase::Deref(ref_id) => CMIRPlaceBase::Deref(cell_map[&ref_id]),
        };
        CMIRPlace { 
            typ: place.typ.monomorphize(bindings), 
            base: low_base, 
            fieldchain: place.fieldchain
        }
    }
}

