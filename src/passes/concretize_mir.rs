use std::collections::{BTreeMap, HashMap, HashSet};

use crate::shared::ids::Id;
use crate::shared::typing::{NewtypeId, TypevarId};
use crate::stages::{mir::*, cmir::*};
use crate::shared::{
    typing::{ConcreteType},
    ids::{BlockId, CellId, FuncId, IdFactory},
    callgraph::get_monomorphizations,
};


pub fn concretize_mir(mir_program: MIRProgram) -> CMIRProgram {
    let MIRProgram { typetable, call_graph, functions, entry } = mir_program;
    let mono_reqs: HashSet<(FuncId, Vec<ConcreteType>)> = get_monomorphizations(&call_graph, &typetable, &entry);

    let mono_func_map: HashMap<(FuncId, Vec<ConcreteType>), FuncId> = mono_reqs
        .into_iter()
        .enumerate()
        .map(|(i,x)| (x, FuncId::from_raw(i)))
        .collect();

    let new_entry = mono_func_map[&(entry, vec![])];

    let mut monomorphizer = Monomorphizer::new(mono_func_map.clone());

    let mono_funcs: HashMap<FuncId, CMIRFunction> = mono_func_map
        .into_iter()
        .map(|((gen_id, tpars), mono_id)| 
            (mono_id, monomorphizer.monomorphize_func(functions[&gen_id].clone(), &tpars)))      // TODO: could probably pop here
        .collect();

    CMIRProgram {
        functions: mono_funcs,
        entry: new_entry,
        newtype_monomorphs: monomorphizer.newtype_monos,
        typetable,
    }
}


struct Monomorphizer {
    mono_func_map: HashMap<(FuncId, Vec<ConcreteType>), FuncId>,
    cell_id_factory: IdFactory<CellId>,
    block_id_factory: IdFactory<BlockId>,
    block_id_map: HashMap<BlockId, BlockId>, // TODO: these are not pushed to!! IMPORTANT!!
    cell_id_map: HashMap<CellId, CellId>,
    newtype_monos: HashSet<(NewtypeId, Vec<ConcreteType>)>,
}
// TODO: maybe later put cell/block-map here, all funcs' ones piled together? unsure yet

impl Monomorphizer {
    
    fn new(mono_func_map: HashMap<(FuncId, Vec<ConcreteType>), FuncId>) -> Self {
        Self { 
            mono_func_map, 
            cell_id_factory: IdFactory::new(),
            block_id_factory: IdFactory::new(),
            block_id_map: HashMap::new(),
            cell_id_map: HashMap::new(),
            newtype_monos: [].into(),
        }
    }

    fn monomorphize_func(&mut self, gen_func: MIRFunction, tparams: &[ConcreteType]) -> CMIRFunction {
        let tparam_bindings: BTreeMap<TypevarId, ConcreteType> = gen_func.typvars
            .into_iter()
            .zip(tparams.iter().cloned())
            .collect();

        // maps old ID -> (new id, mono-d type)
        let cell_map: HashMap<CellId, (CellId, ConcreteType)> = gen_func.cells
            .into_iter()
            .map(|(old_id, gen_typ)| (old_id, (self.cell_id_factory.next_id(), gen_typ.monomorphize(&tparam_bindings))))
            .collect();

        self.cell_id_map = cell_map.clone().into_iter().map(|(old_id, (new_id, _))| (old_id, new_id)).collect(); 

        let block_id_map: HashMap<BlockId, BlockId> = gen_func.blocks
            .keys()
            .map(|old_id| (*old_id, self.block_id_factory.next_id()))
            .collect();

        self.block_id_map = block_id_map.clone();

        let mono_blocks: HashMap<BlockId, CMIRBlock> = gen_func.blocks
            .into_iter()
            .map(|(old_id, block)| (block_id_map[&old_id] , self.monomorphize_block(block, &tparam_bindings)))
            .collect();

        CMIRFunction {
            name: gen_func.name,
            args: gen_func.args.into_iter().map(|old_id| cell_map[&old_id].0).collect(),
            cells: cell_map.into_values().collect(),
            blocks: mono_blocks,
            entry: block_id_map[&gen_func.entry],
            ret_type: gen_func.ret_type.monomorphize(&tparam_bindings),
        }
    }

    fn monomorphize_block(
        &mut self, gen_block: MIRBlock, tparam_bindings: &BTreeMap<TypevarId, ConcreteType>) -> CMIRBlock {
        CMIRBlock {
            statements: gen_block.statements.into_iter().map(|stmt| self.monomorphize_statement(stmt, tparam_bindings)).collect(),
            terminator: self.monomorphize_terminator(gen_block.terminator, tparam_bindings),
        }
    }

    fn monomorphize_statement(&mut self, gen_stmt: MIRStatement, tparam_bindings: &BTreeMap<TypevarId, ConcreteType>) -> CMIRStatement {
        match gen_stmt {
            MIRStatement::Assign { target, value } => CMIRStatement::Assign {
                target: self.monomorphize_place(target, tparam_bindings), 
                value: self.monomorphize_value(value, tparam_bindings),
            },
            MIRStatement::BinOp { target, op, left, right } => CMIRStatement::BinOp { 
                target: self.monomorphize_place(target, tparam_bindings), 
                op, 
                left: self.monomorphize_value(left, tparam_bindings),
                right: self.monomorphize_value(right, tparam_bindings),
            },
            MIRStatement::Call { target, func, type_params, args } => {
                let mono_func = self.mono_func_map[&(func, type_params.into_iter().map(|tpar| tpar.monomorphize(tparam_bindings)).collect())];
                CMIRStatement::Call { 
                    target: self.monomorphize_place(target, tparam_bindings), 
                    func: mono_func,
                    args: args.into_iter().map(|arg| self.monomorphize_value(arg, tparam_bindings)).collect(),
                }
            }
            MIRStatement::Print(value) => CMIRStatement::Print(self.monomorphize_value(value, tparam_bindings))
        }
    }

    fn monomorphize_terminator(&mut self, gen_term: MIRTerminator, tparam_bindings: &BTreeMap<TypevarId, ConcreteType>, ) -> CMIRTerminator {
        match gen_term {
            MIRTerminator::Goto(id) => CMIRTerminator::Goto(self.block_id_map[&id]),
            MIRTerminator::Branch { condition, then_, else_ } => {
                CMIRTerminator::Branch { 
                    condition: self.monomorphize_value(condition, tparam_bindings),
                    then_: self.block_id_map[&then_],
                    else_: self.block_id_map[&else_],
                }
            },
            MIRTerminator::Return(val) => {
                CMIRTerminator::Return(val.map(|some_val| self.monomorphize_value(some_val, tparam_bindings)))
            }
        }
    }

    fn monomorphize_place(&self, gen_place: MIRPlace, tparam_bindings: &BTreeMap<TypevarId, ConcreteType>) -> CMIRPlace {
        let mono_place_base = match gen_place.base {
            MIRPlaceBase::Cell(id) => {
                CMIRPlaceBase::Cell(self.cell_id_map[&id])
            },
            MIRPlaceBase::Deref(ref_id) => CMIRPlaceBase::Deref(self.cell_id_map[&ref_id]),
        };
        CMIRPlace { 
            typ: gen_place.typ.monomorphize(tparam_bindings), 
            base: mono_place_base,
            fieldchain: gen_place.fieldchain,
        }
    }

    fn monomorphize_value(&mut self, gen_val: MIRValue, tparam_bindings: &BTreeMap<TypevarId, ConcreteType>) -> CMIRValue {
        let mono_val_kind = match gen_val.value {
            MIRValueKind::Place(place) => {
                CMIRValueKind::Place(self.monomorphize_place(place, tparam_bindings))
            },
            MIRValueKind::IntLiteral(num) => CMIRValueKind::IntLiteral(num),
            MIRValueKind::BoolTrue => CMIRValueKind::BoolTrue,
            MIRValueKind::BoolFalse => CMIRValueKind::BoolFalse,
            MIRValueKind::StructLiteral { fields } => {
                CMIRValueKind::StructLiteral { 
                    fields: fields
                        .into_iter()
                        .map(|(name, val)| (name, self.monomorphize_value(val, tparam_bindings)))
                        .collect()
                }
            }
            MIRValueKind::Reference(place) => {
                CMIRValueKind::Reference(self.monomorphize_place(place, tparam_bindings))
            }
        };
        let typ = gen_val.typ.monomorphize(tparam_bindings); 
        if let ConcreteType::NewType(id, params) = typ.clone() {
            self.newtype_monos.insert((id, params));
        }
        CMIRValue { 
            typ, 
            value: mono_val_kind,
        }
    }
}
