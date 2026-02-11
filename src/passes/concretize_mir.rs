use std::collections::{BTreeMap, HashMap, HashSet};
use anyhow::{Context, Result, anyhow};


use crate::{
    stages::{mir::*, cmir::*},
    shared::{
        typing::ConcreteType,
        definitions::{Id, BlockId, CellId, FuncId, IdFactory, NewtypeId, TypevarId},
        callgraph::get_monomorphizations,
    }
};


pub fn concretize_mir(mir_program: MIRProgram) -> Result<CMIRProgram> {
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
        .map(|((gen_id, tpars), mono_id)| {
            let func = functions[&gen_id].clone();
            let func_name = func.name.clone();
            Ok((mono_id, monomorphizer
                .monomorphize_func(functions[&gen_id].clone(), &tpars)
                .with_context(|| format!("Failed to monomorphize func {:?}", func_name))?
            ))
        })      // TODO: could probably pop here
        .collect::<Result<HashMap<_,_>>>()?;


    Ok(CMIRProgram {
        functions: mono_funcs,
        entry: new_entry,
        newtype_monomorphs: monomorphizer.newtype_monos,
        typetable,
    })
}


struct Monomorphizer {
    mono_func_map: HashMap<(FuncId, Vec<ConcreteType>), FuncId>,
    cell_id_factory: IdFactory<CellId>,
    block_id_factory: IdFactory<BlockId>,
    block_id_map: HashMap<BlockId, BlockId>, // TODO: these are not pushed to!! IMPORTANT!!
    cell_id_map: HashMap<CellId, CellId>,   // old id -> new id
    cells: HashMap<CellId, ConcreteType>,
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
            cells: HashMap::new(),
            newtype_monos: [].into(),
        }
    }

    fn monomorphize_func(&mut self, gen_func: MIRFunction, tparams: &[ConcreteType]) -> Result<CMIRFunction> {
        let tparam_bindings: BTreeMap<TypevarId, ConcreteType> = gen_func.typvars
            .into_iter()
            .zip(tparams.iter().cloned())
            .collect();

        for (old_cell_id, gen_type) in gen_func.cells {
            let new_id = self.cell_id_factory.next_id();
            self.cell_id_map.insert(old_cell_id, new_id);
            self.cells.insert(new_id, gen_type.monomorphize(&tparam_bindings)?);
        }

        self.block_id_map = gen_func.blocks
            .keys()
            .map(|old_id| (*old_id, self.block_id_factory.next_id()))
            .collect();

        let mono_blocks: HashMap<BlockId, CMIRBlock> = gen_func.blocks
            .into_iter()
            .map(|(old_id, block)| {
                self.monomorphize_block(block, &tparam_bindings)
                    .with_context(|| format!("Failed to monomorphize func {:?}", gen_func.name))
                    .map(|mono_block| (self.block_id_map[&old_id], mono_block))
            })
            .collect::<Result<HashMap<_,_>>>()?;


        Ok(CMIRFunction {
            name: gen_func.name,
            args: gen_func.args.into_iter().map(|old_id| self.cell_id_map[&old_id]).collect(),
            cells: self.cells.clone(),
            blocks: mono_blocks,
            entry: self.block_id_map[&gen_func.entry],
            ret_type: gen_func.ret_type.monomorphize(&tparam_bindings)?,
        })
    }

    fn monomorphize_block(
        &mut self, gen_block: MIRBlock, tparam_bindings: &BTreeMap<TypevarId, ConcreteType>) -> Result<CMIRBlock> {
        Ok(CMIRBlock {
            statements: gen_block.statements.into_iter().map(|stmt| self.monomorphize_statement(stmt, tparam_bindings)).collect::<Result<Vec<_>>>()?,
            terminator: self.monomorphize_terminator(gen_block.terminator, tparam_bindings)?,
        })
    }

    fn monomorphize_statement(&mut self, gen_stmt: MIRStatement, tparam_bindings: &BTreeMap<TypevarId, ConcreteType>) -> Result<CMIRStatement> {
        match gen_stmt {
            MIRStatement::Assign { target, value } => Ok(CMIRStatement::Assign {
                target: self.monomorphize_place(target, tparam_bindings)?, 
                value: self.monomorphize_value(value, tparam_bindings)?,
            }),
            MIRStatement::BinOp { target, op, left, right } => Ok(CMIRStatement::BinOp { 
                target: self.monomorphize_place(target, tparam_bindings)?, 
                op, 
                left: self.monomorphize_value(left, tparam_bindings)?,
                right: self.monomorphize_value(right, tparam_bindings)?,
            }),
            MIRStatement::Call { target, func, type_params, args } => {
                let sgn = (func, type_params.into_iter().map(|tpar| tpar.monomorphize(tparam_bindings)).collect::<Result<Vec<_>>>()?);
                Ok(CMIRStatement::Call { 
                    target: self.monomorphize_place(target, tparam_bindings)?, 
                    func: *self.mono_func_map
                        .get(&sgn)
                        .ok_or_else(|| anyhow!("Function with signature {:?} not found during monomorphization", sgn))?,
                    args: args.into_iter().map(|arg| self.monomorphize_value(arg, tparam_bindings)).collect::<Result<Vec<_>>>()?,
                })
            }
            MIRStatement::Print(value) => Ok(CMIRStatement::Print(self.monomorphize_value(value, tparam_bindings)?))
        }
    }

    fn monomorphize_terminator(&mut self, gen_term: MIRTerminator, tparam_bindings: &BTreeMap<TypevarId, ConcreteType>, ) -> Result<CMIRTerminator> {
        match gen_term {
            MIRTerminator::Goto(id) => Ok(CMIRTerminator::Goto(self.block_id_map[&id])),
            MIRTerminator::Branch { condition, then_, else_ } => {
                Ok(CMIRTerminator::Branch { 
                    condition: self.monomorphize_value(condition, tparam_bindings)?,
                    then_: self.block_id_map[&then_],
                    else_: self.block_id_map[&else_],
                })
            },
            MIRTerminator::Return(val) => {
                Ok(CMIRTerminator::Return(val.map(|some_val| self.monomorphize_value(some_val, tparam_bindings)).transpose()?))
            }
        }
    }

    fn monomorphize_place(&self, gen_place: MIRPlace, tparam_bindings: &BTreeMap<TypevarId, ConcreteType>) -> Result<CMIRPlace> {
        let mono_place_base = match gen_place.base {
            MIRPlaceBase::Cell(id) => {
                CMIRPlaceBase::Cell(self.cell_id_map[&id])
            },
            MIRPlaceBase::Deref(ref_id) => CMIRPlaceBase::Deref(self.cell_id_map[&ref_id]),
        };
        Ok(CMIRPlace { 
            typ: gen_place.typ.monomorphize(tparam_bindings)?, 
            base: mono_place_base,
            fieldchain: gen_place.fieldchain,
        })
    }

    fn monomorphize_value(&mut self, gen_val: MIRValue, tparam_bindings: &BTreeMap<TypevarId, ConcreteType>) -> Result<CMIRValue> {
        let mono_val_kind = match gen_val.value {
            MIRValueKind::Place(place) => {
                CMIRValueKind::Place(self.monomorphize_place(place, tparam_bindings)?)
            },
            MIRValueKind::IntLiteral(num) => CMIRValueKind::IntLiteral(num),
            MIRValueKind::BoolTrue => CMIRValueKind::BoolTrue,
            MIRValueKind::BoolFalse => CMIRValueKind::BoolFalse,
            MIRValueKind::StructLiteral { fields } => CMIRValueKind::StructLiteral { 
                fields: fields
                    .into_iter()
                    .map(|(name, val)| {
                        anyhow::Ok((name, self.monomorphize_value(val, tparam_bindings)?))
                    })
                    .collect::<Result<HashMap<_,_>>>()?
            },      
            MIRValueKind::Reference(place) => {

                CMIRValueKind::Reference(self.monomorphize_place(place, tparam_bindings)?)
            }
        };
        let typ = gen_val.typ.monomorphize(tparam_bindings)?; 
        if let ConcreteType::NewType(id, params) = typ.clone() {
            self.newtype_monos.insert((id, params));
        }
        Ok(CMIRValue { 
            typ, 
            value: mono_val_kind,
        })
    }
}
