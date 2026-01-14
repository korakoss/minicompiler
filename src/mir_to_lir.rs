use std::collections::HashMap;

use crate::stages::common::*;
use crate::stages::lir::*;
use crate::stages::mir::*;
use crate::shared::typing::*;


struct CallArgTable {
    arg_sizes: HashMap< FuncId, Vec<usize>>,
}

impl CallArgTable {

    fn make(sgns: HashMap<FuncId, FuncSignature>, layouts: &LayoutTable) -> CallArgTable {
        let mut arg_sizes: HashMap< FuncId, Vec<usize>> = HashMap::new();
        for (id, sgn) in sgns {
            let mut func_arg_sizes: Vec<usize> = Vec::new();
            for argt in sgn.argtypes.iter() {
                func_arg_sizes.push(layouts.get_layout(argt.clone()).size());
            }
            arg_sizes.insert(id, func_arg_sizes);
            
        }
        CallArgTable { arg_sizes}
    }

    fn table_size(&self, id: FuncId) -> usize {
        self.arg_sizes[&id].iter().sum()
    }

    fn get_offset(&self, id: FuncId, index: usize) -> usize {
        self.arg_sizes[&id][..index].iter().sum()
    }
}


pub struct LIRBuilder {
    cell_chunk_map: HashMap<CellId, (ChunkId, Type)>,
    curr_chunks: HashMap<ChunkId, Chunk>,
    layouts: LayoutTable,
    chunk_counter: usize,
    typetable: TypeTable,
    calltable: CallArgTable
}

impl LIRBuilder {
    
    pub fn lower_mir(program: MIRProgram) -> LIRProgram {
        let layouts = LayoutTable::make(program.typetable.clone());

        let func_sgns: HashMap<FuncId, FuncSignature> = program.functions
            .iter()
            .map(|(id, func)| (*id, func.sgn.0.clone()))
            .collect();

        let calltable = CallArgTable::make(func_sgns, &layouts);

        let mut builder = LIRBuilder {
            cell_chunk_map: HashMap::new(),
            curr_chunks: HashMap::new(),
            layouts,
            chunk_counter: 0,
            typetable: program.typetable,
            calltable: calltable
        };
        LIRProgram {
            functions: program.functions
.into_iter()
                .map(|(id, func)| (id, builder.lower_function(id, func)))
                .collect(),
            entry: program.entry
        }
    }

    fn lower_function(&mut self, id: FuncId,func: MIRFunction) -> LIRFunction {
        self.cell_chunk_map = HashMap::new();   // TODO: assignment could be inlined 
        for (id, cell) in func.cells {
            self.lower_cell(id, cell);
        }
        let arg_structp_chunk = self.add_chunk_to_frame(Chunk { size: 8});
        let retp_chunk = self.add_chunk_to_frame(Chunk { size: 8});       
        
        let arg_chunks: Vec<ChunkId> = func.args
                .into_iter()
                .map(|cell_id| self.cell_chunk_map[&cell_id].0.clone())
                .collect();

        let mut lowered_blocks: HashMap<BlockId, LIRBlock> = func.blocks
                .into_iter()
                .map(|(id, block)| (id, self.lower_block(block)))
                .collect();


        let header_stmts = self.make_func_header(id, arg_chunks.clone());
        lowered_blocks.get_mut(&func.entry).unwrap().statements.splice(0..0, header_stmts);

        LIRFunction {
            blocks: lowered_blocks,
            entry: func.entry,
            chunks: self.curr_chunks.clone(), 
            arg_struct_pointer: arg_structp_chunk,
            ret_pointer: retp_chunk,
            args: arg_chunks,
        }
    }

    fn make_func_header(&mut self, id: FuncId, arg_cells: Vec<ChunkId>) -> Vec<LIRStatement> {
        let arg_structp_chunk = self.add_chunk_to_frame(
            Chunk{size: self.calltable.table_size(id)}
        );

        let mut assign_stmts: Vec<LIRStatement> = Vec::new();
        for (i, arg) in arg_cells.iter().enumerate() {
            let arg_offs = self.calltable.get_offset(id, i);
            let argsize = self.calltable.arg_sizes[&id][i];
            let arg_assignment = LIRStatement::Store { 
                dest: LIRPlace { 
                    size: argsize, 
                    place: LIRPlaceKind::Local { base: *arg, offset: 0}
                }, 
                value: LIRValue { 
                    size: argsize, 
                    value: LIRValueKind::Place(LIRPlace { 
                        size: argsize, 
                        place:  LIRPlaceKind::Deref { 
                            pointer: arg_structp_chunk, 
                            offset: arg_offs, 
                        }
                    })
                }
            };
            assign_stmts.push(arg_assignment);
        }
        assign_stmts
    }

    fn lower_block(&mut self, block: MIRBlock) -> LIRBlock {
        let mut statements: Vec<LIRStatement> = Vec::new();
        for stmt in block.statements {
            let lowered = self.lower_stmt(stmt);
            statements.extend(lowered.into_iter());
        }
        let (terminator, term_stmts) = self.lower_terminator(block.terminator);
        statements.extend(term_stmts.into_iter());
        LIRBlock { statements, terminator }
    }

    fn lower_stmt(&mut self, stmt: MIRStatement) -> Vec<LIRStatement> {
        match stmt {
            MIRStatement::Assign { target, value } => {
                let lir_target = self.lower_place(target);
                self.lower_value_into_place(value, lir_target)
            }
            MIRStatement::BinOp { target, op, left, right } => {
                let lir_target = self.lower_place(target);
                let (left_opnd, left_stmts) = self.lower_value_into_operand(left);
                let (right_opnd, right_stmts) = self.lower_value_into_operand(right);
                let bin_stmt = LIRStatement::BinOp { 
                    dest: lir_target, 
                    op: op, 
                    left: left_opnd, 
                    right: right_opnd 
                };
                [left_stmts, right_stmts, vec![bin_stmt]].concat()
            }
            MIRStatement::Call { target, func, args } => {
                let lir_target = self.lower_place(target);

                let mut arg_stmts_coll: Vec<LIRStatement> = Vec::new();

                let args_struct_chunk = self.add_chunk_to_frame(Chunk { 
                    size: self.calltable.table_size(func)
                });
                for (i, arg) in args.into_iter().enumerate() {
                    let arg_size = self.layouts.get_layout(arg.typ.clone()).size();
                    let arg_offset = self.calltable.get_offset(func, i);
                    let arg_place = LIRPlace {
                        size: arg_size, 
                        place: LIRPlaceKind::Local { 
                            base: args_struct_chunk, 
                            offset: arg_offset, 
                        }
                    };
                    let arg_stmts = self.lower_value_into_place(arg, arg_place);
                    arg_stmts_coll.extend(arg_stmts.into_iter());

                 }
                let lir_call = LIRStatement::Call { 
                    dest: lir_target, 
                    func, 
                    arg_struct_base: args_struct_chunk 
                };
                [arg_stmts_coll, vec![lir_call]].concat()
            }
            MIRStatement::Print(value) => {
                let (opnd, stmts) = self.lower_value_into_operand(value);
                [stmts, vec![LIRStatement::Print(opnd)]].concat()
            }
        }
    }

    fn lower_terminator(&mut self, term: MIRTerminator) -> (LIRTerminator, Vec<LIRStatement>) {
        match term {
            MIRTerminator::Goto(block_id) => (LIRTerminator::Goto { dest: block_id }, Vec::new()),
            MIRTerminator::Branch { condition, then_, else_ } => {
                let (cond_op, cond_stmts) = self.lower_value_into_operand(condition);
                let term = LIRTerminator::Branch { 
                    condition: cond_op, 
                    then_block: then_, 
                    else_block: else_ 
                };
                (term, cond_stmts)
            }
            MIRTerminator::Return(ret_val) => {
                match ret_val {
                    None => (LIRTerminator::Return(None), Vec::new()),
                    Some(value) => {
                        let (retval_op, retval_stmts) = self.lower_value_into_operand(value);
                        (LIRTerminator::Return(Some(retval_op)), retval_stmts)
                    }
                }
            }
        }
    }

    fn lower_value_into_operand(&mut self, value: MIRValue) -> (LIRValue, Vec<LIRStatement>) {
        let size = self.layouts.get_layout(value.typ.clone()).size();
        match value.value {
            MIRValueKind::Place(val_place) => {
                let lir_val_place = self.lower_place(val_place);
                (LIRValue {size, value: LIRValueKind::Place(lir_val_place)}, Vec::new())
            },
            MIRValueKind::IntLiteral(num) => {
                (LIRValue {size, value: LIRValueKind::IntLiteral(num)}, Vec::new())
            },
            MIRValueKind::BoolTrue => {
                (LIRValue {size, value: LIRValueKind::BoolTrue}, Vec::new())
            }
            MIRValueKind::BoolFalse => {
                (LIRValue {size, value: LIRValueKind::BoolFalse}, Vec::new())
            }
            MIRValueKind::StructLiteral {..} => {
                let temp_chunk = Chunk {
                    size: self.layouts.get_layout(value.typ.clone()).size(),
                };
                let temp_id = self.add_chunk_to_frame(temp_chunk);
                let temp_place = LIRPlace {
                    size: size,
                    place: LIRPlaceKind::Local { base: temp_id, offset: 0}
                };

                // Mehh. Maybe add type info back to MIRV?
                let stmts = self.lower_value_into_place(value, temp_place.clone());
                (LIRValue{ size, value: LIRValueKind::Place(temp_place)}, stmts)
            }
            MIRValueKind::Reference(refd) => {
                let refd_place = self.lower_place(refd);
                (LIRValue {size, value: LIRValueKind::Reference(refd_place)}, vec![]) 
            }
        }
    }

    fn lower_value_into_place(&self, value: MIRValue, target: LIRPlace) -> Vec<LIRStatement> {
        let size = self.layouts.get_layout(value.typ.clone()).size();
        match value.value {
            MIRValueKind::Place(val_place) => {
                let lir_val_place = self.lower_place(val_place);
                vec![LIRStatement::Store{dest: target, value: LIRValue { size, value: LIRValueKind::Place(lir_val_place)}}] 
            },
            MIRValueKind::IntLiteral(num) => {
                vec![LIRStatement::Store{dest: target, value: LIRValue{ size, value: LIRValueKind::IntLiteral(num)}}]
            },
            MIRValueKind::BoolTrue => {
                vec![LIRStatement::Store{dest: target, value: LIRValue { size, value: LIRValueKind::BoolTrue}}]
            }
            MIRValueKind::BoolFalse => {
                vec![LIRStatement::Store{dest: target, value: LIRValue { size, value: LIRValueKind::BoolFalse}}]
            }
            MIRValueKind::StructLiteral { typ, fields } => {
                let LayoutInfo::Struct { size, field_offsets } = self.layouts.get_layout(typ) else {
                    unreachable!();
                };
                let mut stmts: Vec<LIRStatement> = Vec::new();
                for (fname, fexpr) in fields {
                    let fsize = self.layouts.get_layout(fexpr.typ.clone()).size();
                    let f_target = LIRPlace {
                        size: fsize,
                        place: increment_place_offset(target.place.clone(), field_offsets[&fname]),
                    };
                    stmts.extend(self.lower_value_into_place(fexpr, f_target));
                }
                stmts
            }
            MIRValueKind::Reference(refd) => {
                let refd_place = self.lower_place(refd);
                let stmt = LIRStatement::Store { dest: target, value: LIRValue { size, value: LIRValueKind::Reference(refd_place)}}; 
                vec![stmt]
            }
        }
    }

    fn lower_place(&self, place: MIRPlace) -> LIRPlace {
        // TODO: weird solution, change it
        let size = self.layouts.get_layout(place.typ).size();
        match place.base {
            MIRPlaceBase::Cell(c_id) => {

                let base_type = self.cell_chunk_map[&c_id].1.clone();
                let (final_offset, final_type) = self.lower_fieldchain(base_type, place.fieldchain);
                LIRPlace {
                    size,
                    place: LIRPlaceKind::Local{
                        base: self.cell_chunk_map[&c_id].0, 
                        offset: final_offset 
                    }
                }
            },
            MIRPlaceBase::Deref(c_id) => {
                let ref_type = self.cell_chunk_map[&c_id].1.clone();
                let Type::Reference(deref_type) = ref_type else {unreachable!()};
                let (final_offset, final_type) = self.lower_fieldchain(*deref_type, place.fieldchain);
                LIRPlace {
                    size,
                    place: LIRPlaceKind::Deref { 
                        pointer: self.cell_chunk_map[&c_id].0, 
                        offset: final_offset,
                    }
                }
            }
        }
    }

    fn lower_fieldchain(&self, base_type: Type, chain: Vec<String>) -> (usize, Type) {
        let mut curr_typ = base_type;
        let mut curr_offset = 0;
        
        for field in chain {
            let curr_typ_layout = self.layouts.get_layout(curr_typ.clone());

            match curr_typ_layout {
                LayoutInfo::Struct { size, field_offsets } => {
                    let TypeDef::NewType(TypeConstructor::Struct { fields }) = self.typetable.get_typedef(curr_typ) else {
                        unreachable!();
                    };
                    curr_typ = fields[&field].clone();
                    curr_offset = curr_offset + field_offsets[&field];
                } 
                LayoutInfo::Primitive(..) => {
                    panic!("This is primitive, shouldn't have a field");
                }
            }
        }
        (curr_offset, curr_typ)
    }
    
    fn lower_cell(&mut self, id: CellId, cell: Cell) {
        // TODO: This should lower into LIRPlace. I think?
        let chunk = Chunk {size: self.layouts.get_layout(cell.typ.clone()).size()};
        let chunk_id = self.add_chunk_to_frame(chunk);
        self.cell_chunk_map.insert(id, (chunk_id, cell.typ));
    }

    fn add_chunk_to_frame(&mut self, chunk: Chunk) -> ChunkId {
        let chunk_id = ChunkId(self.chunk_counter);
        self.chunk_counter = self.chunk_counter + 1;
        self.curr_chunks.insert(chunk_id, chunk);
        chunk_id
    }

}


fn increment_place_offset(place: LIRPlaceKind, increment: usize) -> LIRPlaceKind {
    match place {
        LIRPlaceKind::Local { base, offset } => LIRPlaceKind::Local { base, offset: offset + increment },
        LIRPlaceKind::Deref { pointer, offset } => LIRPlaceKind::Deref { pointer, offset: offset + increment},
    }
}


#[derive(Clone, Debug)]
pub enum LayoutInfo {
    Primitive(usize),               // Variable size
    Struct {
        size: usize,
        field_offsets: HashMap<String, usize>
    }
}

impl LayoutInfo {
    pub fn size(&self) -> usize {
        match self {
            &LayoutInfo::Primitive(size) => size,
            &LayoutInfo::Struct{size, ..} => size,
        }
    }
}


#[derive(Clone, Debug)]
pub struct LayoutTable {
    newtype_layouts: HashMap<TypeIdentifier, LayoutInfo>
}

impl LayoutTable {

    pub fn make(typetable: TypeTable) -> LayoutTable {
        let mut table = LayoutTable{newtype_layouts: HashMap::new()};
        for tp_id in typetable.topo_order {
            let tp_constr = typetable.newtype_map[&tp_id].clone();
            table.newtype_layouts.insert(tp_id, table.make_newtype_layout(tp_constr));
        }
        table
    }   

    pub fn get_layout(&self, typ: Type) -> LayoutInfo {
        match typ {
            Type::Prim(prim_tp) => self.get_primitive_layout(prim_tp),
            Type::NewType(tp_constr) => self.newtype_layouts[&tp_constr].clone(),
            Type::Reference(..) => LayoutInfo::Primitive(8)
        }
    }

    fn get_primitive_layout(&self, prim_tp: PrimType) -> LayoutInfo {
        LayoutInfo::Primitive(8)        // Temporarily so; update later
    }
    
    fn make_newtype_layout(&self, deriv_typ: TypeConstructor) -> LayoutInfo {
        
        let TypeConstructor::Struct{fields} = deriv_typ else {
            unimplemented!();
        };

        let mut f_offsets: HashMap<String, usize> = HashMap::new();

        let mut curr_offset = 0;
for (fname, ftype) in fields {
            f_offsets.insert(fname, curr_offset);
            let fsize = self.get_layout(ftype).size(); 
            curr_offset = curr_offset + fsize;
        }

        LayoutInfo::Struct { 
            size: curr_offset, 
            field_offsets: f_offsets 
        }
    }
}

