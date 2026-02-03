use std::collections::{BTreeMap, HashMap};

use crate::shared::typing::{NewtypeId, TypevarId};
use crate::stages::{mir::*, cmir::*};
use crate::shared::{
    typing::{ConcreteType},
    tables::{GenericTypetable},
    ids::{BlockId, CellId, FuncId, IdFactory}
};



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


