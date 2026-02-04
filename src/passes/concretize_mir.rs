use std::collections::{BTreeMap, HashMap, HashSet};

use crate::shared::ids::Id;
use crate::shared::typing::{NewtypeId, TypevarId};
use crate::stages::{mir::*, cmir::*};
use crate::shared::{
    typing::{ConcreteType},
    tables::{GenericTypetable},
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

    let monomorphizer = Monomorphizer::new(mono_func_map.clone());

    let mono_funcs: HashMap<FuncId, CMIRFunction> = mono_func_map
        .into_iter()
        .map(|((gen_id, tpars), mono_id)| 
            (mono_id, monomorphizer.monomorphize_func(functions[&gen_id].clone(), tpars)))      // TODO: could probably pop here
        .collect();

    CMIRProgram {
        functions: mono_funcs,
        entry: new_entry,
    }
}


struct Monomorphizer {
    mono_func_map: HashMap<(FuncId, Vec<ConcreteType>), FuncId>,
}

impl Monomorphizer {
    
    fn new(mono_func_map: HashMap<(FuncId, Vec<ConcreteType>), FuncId>) -> Self {
        Self { mono_func_map }
    }

    fn monomorphize_func(&self, gen_func: MIRFunction, tparams: Vec<ConcreteType>) -> CMIRFunction {
        unimplemented!();
    }

}

fn monomorphize_function(gen_func: MIRFunction, tparams: Vec<ConcreteType>) -> CMIRFunction {
    let mono_func = CMIRFunction {
        name: gen_func.name,
        args: unimplemented!(),
        cells: unimplemented!(),
        blocks: unimplemented!(),
        entry: unimplemented!(),
        ret_type: unimplemented!(),
    };
    unimplemented!();
}

pub struct MIRLowerer {
    generic_functions: HashMap<FuncId, MIRFunction>,
    mono_types: Vec<(NewtypeId, Vec<ConcreteType>)>,
    typetable: GenericTypetable,
    blockid_factory: IdFactory<BlockId>,
    cellid_factory: IdFactory<CellId>,
    funcid_factory: IdFactory<FuncId>,
 }


fn lower_terminator(
    term:MIRTerminator, 
    block_map: &HashMap<BlockId, BlockId>,
    bindings: &BTreeMap<TypevarId, ConcreteType>,
    cell_map: &HashMap<CellId, CellId>
) -> CMIRTerminator {
    match term {
        MIRTerminator::Goto(id) => CMIRTerminator::Goto(block_map[&id]),
        MIRTerminator::Branch { condition, then_, else_ } => {
            CMIRTerminator::Branch { 
                condition: lower_value(condition, bindings, cell_map),
                then_: block_map[&then_],
                else_: block_map[&else_],
            }
        }
        MIRTerminator::Return(val) => {
            CMIRTerminator::Return(val.map(|some_val| lower_value(some_val, bindings, cell_map)))
        }
    }
}


fn lower_value(
    val: MIRValue, 
    bindings: &BTreeMap<TypevarId, ConcreteType>,
    cell_map: &HashMap<CellId, CellId>
) -> CMIRValue {
    let low_value = match val.value {
        MIRValueKind::Place(place) => {
            CMIRValueKind::Place(lower_place(place, bindings, cell_map))
        }
        MIRValueKind::IntLiteral(num) => CMIRValueKind::IntLiteral(num),
        MIRValueKind::BoolTrue => CMIRValueKind::BoolTrue,
        MIRValueKind::BoolFalse => CMIRValueKind::BoolFalse,
        MIRValueKind::StructLiteral { fields } => {
            CMIRValueKind::StructLiteral { 
                fields: fields
                    .into_iter()
                    .map(|(name, val)| (name, lower_value(val, bindings, cell_map)))
                    .collect()
            }
        }
        MIRValueKind::Reference(place) => {
            CMIRValueKind::Reference(lower_place(place, bindings, cell_map))
        }
    };
    CMIRValue { 
        typ: val.typ.monomorphize(bindings), 
        value: low_value 
    }
}


fn lower_place(
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


