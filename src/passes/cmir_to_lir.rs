use std::{collections::{HashMap, HashSet}, usize};

use crate::{
    shared::{
        ids::{CellId, IdFactory}, tables::{ConcreteShape, GenericTypetable}, typing::{ConcreteType, NewtypeId}
    }, 
    stages::{
        cmir::*,
        lir::*,
    },
};


struct LayoutTable {
    layouts: HashMap<(NewtypeId, Vec<ConcreteType>), ChunkLayout>,
}

impl LayoutTable {

    fn make(typetable: GenericTypetable ,concrete_newtypes: HashSet<(NewtypeId, Vec<ConcreteType>)>) -> Self {
        let mut table = Self { layouts: HashMap::new() };
        let mut concrete_newtypes = concrete_newtypes
            .into_iter()
            .collect::<Vec<_>>();
        concrete_newtypes.sort_by_key(|(id, typ)| typetable.get_genericity_rank(&ConcreteType::NewType(id.clone(), typ.clone())));
        for (id, tparams) in concrete_newtypes {
            let type_shape = typetable.monomorphize(id.clone(), tparams.clone());
            let type_layout = match type_shape {
                ConcreteShape::Struct { fields } => {
                    let fields: Vec<(String, ConcreteType)> = fields.into_iter().collect();
                    ChunkLayout {
                        size: fields.iter().map(|(_ ,ftyp)| table.get_layout(&ftyp).size).sum(),
                        typ: ConcreteType::NewType(id.clone(), tparams.clone()),
                        kind: LayoutKind::Struct(fields),
                    }
                },
                ConcreteShape::Enum {..} => {
                    unimplemented!();
                },
            };
            table.layouts.insert((id, tparams), type_layout);
        }
        table
    }

    fn get_layout(&self, typ: &ConcreteType) -> ChunkLayout {
        match typ {
            ConcreteType::Prim(..) => ChunkLayout { size: 8, typ: typ.clone(), kind: LayoutKind::Atomic },
            ConcreteType::Reference(..) => ChunkLayout { size: 8, typ: typ.clone(), kind: LayoutKind::Atomic },
            ConcreteType::NewType(id, tparams) => self.layouts[&(id.clone(), tparams.clone())].clone(),
        }
    }
}


#[derive(Clone, Debug)]
pub struct ChunkLayout {
    size: usize,                        // NOTE: maybe align etc here too
    typ: ConcreteType,
    kind: LayoutKind,
}

#[derive(Clone, Debug)]
pub enum LayoutKind {
    Atomic,                             // primitives, pointers -- no internal structure basically
    Struct(Vec<(String, ConcreteType)>),     // it's been given a fixed order
}




pub fn lower_cmir(program: CMIRProgram) -> LIRProgram {
    let layout_table = LayoutTable::make(program.typetable, program.newtype_monomorphs);
    let mut builder = LIRBuilder::new(layout_table);
    LIRProgram { 
        functions: program.functions
            .into_iter()
            .map(|(id, func)| (id, builder.lower_function(func)))
            .collect(), 
        entry: program.entry,
    }
}



struct LIRBuilder {
    layout_table: LayoutTable,
    cell_chunk_map: HashMap<CellId, CellId>,
    chunk_table: HashMap<CellId, ChunkLayout>,
    chunk_id_factory: IdFactory<CellId>,
}

impl LIRBuilder {

    fn new(layout_table: LayoutTable) -> Self {
        Self { 
            layout_table,
            cell_chunk_map: HashMap::new(),
            chunk_table: HashMap::new(), 
            chunk_id_factory: IdFactory::new(),
        }
    }

    fn lower_function(&mut self, func: CMIRFunction) -> LIRFunction {
        for (cell_id, cell_type) in func.cells.iter() {
            let chunk_id = self.chunk_id_factory.next_id();
            self.cell_chunk_map.insert(*cell_id, chunk_id);
            self.chunk_table.insert(chunk_id, self.layout_table.get_layout(cell_type));
        }
        LIRFunction {
            blocks: func.blocks
                .into_iter()
                .map(|(id, block)| (id, self.lower_block(block)))
                .collect(),
            entry: func.entry, 
            chunks: func.cells
                .into_keys()
                .map(|id| (id, self.chunk_table[&self.cell_chunk_map[&id]].size))
                .collect(),
            args: func.args
                .into_iter()
                .map(|id| self.cell_chunk_map[&id])
                .collect(),
        }
    }

    fn lower_block(&mut self, block: CMIRBlock) -> LIRBlock {
        let mut statements: Vec<LIRStatement> = Vec::new();
        for stmt in block.statements {
            let lowered = self.lower_stmt(stmt);
            statements.extend(lowered);
        }
        let (terminator, term_stmts) = self.lower_terminator(block.terminator);
        statements.extend(term_stmts);
        LIRBlock { statements, terminator }
    }

    fn lower_stmt(&mut self, stmt: CMIRStatement) -> Vec<LIRStatement> {
        match stmt {
            CMIRStatement::Assign { target, value } => {
                let lir_target = self.lower_place(target);
                self.lower_value_into_place(value, lir_target)
            }
            CMIRStatement::BinOp { target, op, left, right } => {
                let (left_opnd, left_stmts) = self.lower_value(left);
                let (right_opnd, right_stmts) = self.lower_value(right);
                let bin_stmt = LIRStatement::BinOp { 
                    dest: self.lower_place(target),
                    op,
                    left: left_opnd, 
                    right: right_opnd 
                };
                [left_stmts, right_stmts, vec![bin_stmt]].concat()
            }
            CMIRStatement::Call { target, func, args } => {
                let mut arg_places: Vec<LIRPlace> = Vec::new();
                let mut arg_stmts_coll: Vec<LIRStatement> = Vec::new();
                for arg in args {
                    let arg_place = LIRPlace {
                        size: self.layout_table.get_layout(&arg.typ).size,
                        place: LIRPlaceKind::Local { 
                            base: self.add_temp_chunk(&arg.typ),
                            offset: 0, 
                        }
                    };
                    arg_places.push(arg_place.clone());
                    arg_stmts_coll.extend(self
                        .lower_value_into_place(arg, arg_place)
                        .into_iter()
                    );
                }
                let lir_call = LIRStatement::Call { 
                    dest: self.lower_place(target),
                    func, 
                    args: arg_places
                };
                [arg_stmts_coll, vec![lir_call]].concat()
            }
            CMIRStatement::Print(value) => {
                let (opnd, stmts) = self.lower_value(value);
                [stmts, vec![LIRStatement::Print(opnd)]].concat()
            }
        }
    }


    fn lower_terminator(&mut self, term: CMIRTerminator) -> (LIRTerminator, Vec<LIRStatement>) {
        match term {
            CMIRTerminator::Goto(block_id) => (LIRTerminator::Goto { dest: block_id }, Vec::new()),
            CMIRTerminator::Branch { condition, then_, else_ } => {
                let (cond_op, cond_stmts) = self.lower_value(condition);
                let term = LIRTerminator::Branch { 
                    condition: cond_op, 
                    then_block: then_, 
                    else_block: else_ 
                };
                (term, cond_stmts)
            }
            CMIRTerminator::Return(ret_val) => {
                match ret_val {
                    None => (LIRTerminator::Return(None), Vec::new()),
                    Some(value) => {
                        let (retval_op, retval_stmts) = self.lower_value(value);
                        (LIRTerminator::Return(Some(retval_op)), retval_stmts)
                    }
                }
            }
        }
    }
        
   fn lower_value(&mut self, value: CMIRValue) -> (LIRValue, Vec<LIRStatement>) {
        let size = self.layout_table.get_layout(&value.typ).size;
        match value.value {
            CMIRValueKind::Place(val_place) => {
                let lir_val_place = self.lower_place(val_place);
                (LIRValue {size, value: LIRValueKind::Place(lir_val_place)}, Vec::new())
            },
            CMIRValueKind::IntLiteral(num) => {
                (LIRValue {size, value: LIRValueKind::IntLiteral(num)}, Vec::new())
            },
            CMIRValueKind::BoolTrue => {
                (LIRValue {size, value: LIRValueKind::BoolTrue}, Vec::new())
            }
            CMIRValueKind::BoolFalse => {
                (LIRValue {size, value: LIRValueKind::BoolFalse}, Vec::new())
            }
            CMIRValueKind::StructLiteral {..} => {
                let temp_chunk_id = self.add_temp_chunk(&value.typ);
                let temp_place = LIRPlace {
                    size,
                    place: LIRPlaceKind::Local { base: temp_chunk_id, offset: 0}
                };
                let stmts = self.lower_value_into_place(value, temp_place.clone());
                (LIRValue{ size, value: LIRValueKind::Place(temp_place)}, stmts)
            }
            CMIRValueKind::Reference(refd) => {
                let refd_place = self.lower_place(refd);
                (LIRValue {size, value: LIRValueKind::Reference(refd_place)}, vec![]) 
            }
        }
    }

    fn add_temp_chunk(&mut self, typ: &ConcreteType) -> CellId {
        let id = self.chunk_id_factory.next_id();
        let chunk_layout = self.layout_table.get_layout(typ);
        self.chunk_table.insert(id, chunk_layout);
        id
    }


    fn lower_value_into_place(
        &mut self, 
        value: CMIRValue, 
        target: LIRPlace
    ) -> Vec<LIRStatement> {
        let size = self.layout_table.get_layout(&value.typ).size;
        match value.value {
            CMIRValueKind::Place(val_place) => {
                let lir_val_place = self.lower_place(val_place);
                vec![LIRStatement::Store{dest: target, value: LIRValue { size, value: LIRValueKind::Place(lir_val_place)}}] 
            },
            CMIRValueKind::IntLiteral(num) => {
                vec![LIRStatement::Store{dest: target, value: LIRValue{ size, value: LIRValueKind::IntLiteral(num)}}]
            },
            CMIRValueKind::BoolTrue => {
                vec![LIRStatement::Store{dest: target, value: LIRValue { size, value: LIRValueKind::BoolTrue}}]
            }
            CMIRValueKind::BoolFalse => {
                vec![LIRStatement::Store{dest: target, value: LIRValue { size, value: LIRValueKind::BoolFalse}}]
            }
            CMIRValueKind::StructLiteral { fields } => {
                let mut stmts: Vec<LIRStatement> = Vec::new();
                let mut curr_field_offset = 0;
                for (_, fexpr) in fields {
                    let fsize = self.layout_table.get_layout(&fexpr.typ).size;
                    let f_target = LIRPlace {
                        size: fsize,
                        place: increment_place_offset(target.place.clone(), curr_field_offset),
                    };
                    curr_field_offset += fsize;
                    stmts.extend(self.lower_value_into_place(fexpr, f_target));
                }
                stmts
            }
            CMIRValueKind::Reference(refd) => {
                let refd_place = self.lower_place(refd);
                let stmt = LIRStatement::Store { dest: target, value: LIRValue { size, value: LIRValueKind::Reference(refd_place)}}; 
                vec![stmt]
            }
        }
    }


    fn lower_place(&mut self, place: CMIRPlace) -> LIRPlace {
        let kind = match place.base {
            CMIRPlaceBase::Cell(id) => {
                let base_type = self.chunk_table[&self.cell_chunk_map[&id]].typ.clone();
                let offset = self.lower_field_access_chain(&base_type, &place.fieldchain);
                LIRPlaceKind::Local { base: id, offset}
            },
            CMIRPlaceBase::Deref(ref_id) => {
                let ref_type = self.chunk_table[&self.cell_chunk_map[&ref_id]].typ.clone();
                let ConcreteType::Reference(typ) = ref_type else {unreachable!()};
                let offset = self.lower_field_access_chain(&typ, &place.fieldchain);
                LIRPlaceKind::Deref { pointer: ref_id, offset}
            }
        };
        LIRPlace { 
            size: self.layout_table.get_layout(&place.typ).size, 
            place: kind, 
        }
    } 

    fn lower_field_access_chain(
        &mut self, 
        base_type: &ConcreteType, 
        chain: &[String],
    ) -> usize {
        
        let mut current_type: ConcreteType = base_type.clone();
        let mut current_offset = 0;

        for field_name in chain {
            let ChunkLayout { size: _, typ: _, kind: LayoutKind::Struct(curr_fields) }= self.layout_table.get_layout(&current_type) else {
                panic!("Encountered field access on non-struct type");      // Note: this would be caught earlier
            };
            let field_idx = curr_fields.iter().position(|(fname,_)| fname == field_name).expect("Field name not found in struct");
            let field_offset: usize = curr_fields[..field_idx].iter().map(|(_, ftyp)| self.layout_table.get_layout(ftyp).size).sum();
            current_offset += field_offset;
            current_type = curr_fields[field_idx].1.clone();
        }
        current_offset
    }
}


fn increment_place_offset(place: LIRPlaceKind, increment: usize) -> LIRPlaceKind {
    match place {
        LIRPlaceKind::Local { base, offset } => LIRPlaceKind::Local { base, offset: offset + increment },
        LIRPlaceKind::Deref { pointer, offset } => LIRPlaceKind::Deref { pointer, offset: offset + increment},
    }
}
