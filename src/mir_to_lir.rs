use std::collections::HashMap;

use crate::stages::lir::*;
use crate::stages::mir::*;
use crate::shared::typing::*;


pub struct LIRBuilder {
    curr_cell_vreg_map: HashMap<CellId, VRegId>,
    curr_cells: HashMap<CellId, Cell>,
    layouts: LayoutTable,
    curr_vreg_table: HashMap<VRegId, VRegInfo>,
    vreg_counter: usize,
    typetable: TypeTable,
}

impl LIRBuilder {
    
    pub fn lower_mir(program: MIRProgram) -> LIRProgram {
        let layouts = LayoutTable::make(program.typetable.clone());
        let mut builder = LIRBuilder {
            curr_cell_vreg_map: HashMap::new(),
            curr_cells: HashMap::new(),
            layouts,
            curr_vreg_table: HashMap::new(),
            vreg_counter: 0,
            typetable: program.typetable
        };
        LIRProgram {
            functions: program.functions
.into_iter()
                .map(|(id, func)| (id, builder.lower_function(func)))
                .collect(),
            entry: program.entry
        }
    }

    fn lower_function(&mut self, func: MIRFunction) -> LIRFunction {
        self.curr_cell_vreg_map = HashMap::new();
        self.curr_vreg_table = HashMap::new();
        for (id, cell) in func.cells {
            self.lower_cell(id, cell);
        }
        LIRFunction {
            blocks: func.blocks
                .into_iter()
                .map(|(id, block)| (id, self.lower_block(block)))
                .collect(),
            entry: func.entry,
            vregs: self.curr_vreg_table.clone(),
            args: func.args
                .into_iter()
                .map(|cell_id| self.curr_cell_vreg_map[&cell_id].clone())
                .collect()
        }
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
                let mut arg_places: Vec<LIRPlace> = Vec::new();
                let mut arg_stmts_coll: Vec<LIRStatement> = Vec::new();

                for arg in args {
                    let arg_typ = arg.typ.clone();
                    let arg_vreg = self.add_vreg(VRegInfo{
                        size: self.layouts.get_layout(arg_typ.clone()).size(),
                        align: 8,
                    }); 
                    let arg_place = LIRPlace {
                        typ: arg_typ.clone(),
                        place: LIRPlaceKind::Local { 
                            base: arg_vreg, 
                            offset: 0, 
                        }
                    };
                    arg_places.push(arg_place.clone());
                    let arg_stmts = self.lower_value_into_place(arg, arg_place);
                    arg_stmts_coll.extend(arg_stmts.into_iter());
                }
                let lir_call = LIRStatement::Call { 
                    dest: lir_target, 
                    func, 
                    args: arg_places
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
        let val_typ = value.typ.clone();
        match value.value {
            MIRValueKind::Place(val_place) => {
                let lir_val_place = self.lower_place(val_place);
                (LIRValue {typ: val_typ.clone(), value: LIRValueKind::Place(lir_val_place)}, Vec::new())
            },
            MIRValueKind::IntLiteral(num) => {
                (LIRValue {typ: Type::Prim(PrimType::Integer), value: LIRValueKind::IntLiteral(num)}, Vec::new())
            },
            MIRValueKind::BoolTrue => {
                (LIRValue {typ: Type::Prim(PrimType::Bool), value: LIRValueKind::BoolTrue}, Vec::new())
            }
            MIRValueKind::BoolFalse => {
                (LIRValue {typ: Type::Prim(PrimType::Bool), value: LIRValueKind::BoolFalse}, Vec::new())
            }
            MIRValueKind::StructLiteral {..} => {
                let temp_vreg_info = VRegInfo {
                    size: self.layouts.get_layout(val_typ.clone()).size(),
                    align: 8
                };
                let temp_id = self.add_vreg(temp_vreg_info);
                let temp_place = LIRPlace {
                    typ: val_typ.clone(),
                    place: LIRPlaceKind::Local { base: temp_id, offset: 0}
                };

                // Mehh. Maybe add type info back to MIRV?
                let stmts = self.lower_value_into_place(value, temp_place.clone());
                (LIRValue{ typ: val_typ, value: LIRValueKind::Place(temp_place)}, stmts)
            }
            MIRValueKind::Reference(refd) => {
                let refd_place = self.lower_place(refd);
                (LIRValue { typ: Type::Reference(Box::new(refd_place.typ.clone())), value: LIRValueKind::Reference(refd_place)}, vec![]) 
            }
            MIRValueKind::Dereference(reference) => {
                let ref_vreg = self.curr_cell_vreg_map[&reference].clone();
                let ref_type = self.curr_cells[&reference].typ.clone();
                let Type::Reference(deref_typ) = ref_type else {unreachable!()};
                (LIRValue { typ: *deref_typ, value: LIRValueKind::Dereference(ref_vreg)}, vec![])
            }
        }
    }

    fn lower_value_into_place(&self, value: MIRValue, target: LIRPlace) -> Vec<LIRStatement> {
        match value.value {
            MIRValueKind::Place(val_place) => {
                let lir_val_place = self.lower_place(val_place);
                vec![LIRStatement::Store{dest: target, value: LIRValue { typ: value.typ, value: LIRValueKind::Place(lir_val_place)}}] 
            },
            MIRValueKind::IntLiteral(num) => {
                vec![LIRStatement::Store{dest: target, value: LIRValue{ typ: Type::Prim(PrimType::Integer), value: LIRValueKind::IntLiteral(num)}}]
            },
            MIRValueKind::BoolTrue => {
                vec![LIRStatement::Store{dest: target, value: LIRValue { typ: Type::Prim(PrimType::Bool), value: LIRValueKind::BoolTrue}}]
            }
            MIRValueKind::BoolFalse => {
                vec![LIRStatement::Store{dest: target, value: LIRValue { typ: Type::Prim(PrimType::Bool), value: LIRValueKind::BoolFalse}}]
            }
            MIRValueKind::StructLiteral { typ, fields } => {
                let LayoutInfo::Struct { size, field_offsets } = self.layouts.get_layout(typ) else {
                    unreachable!();
                };
                let mut stmts: Vec<LIRStatement> = Vec::new();
                for (fname, fexpr) in fields {
                    let f_target = LIRPlace {
                        typ: fexpr.typ.clone(),
                        place: increment_place_offset(target.place.clone(), field_offsets[&fname]),
                    };
                    stmts.extend(self.lower_value_into_place(fexpr, f_target));
                }
                stmts
            }
            MIRValueKind::Reference(refd) => {
                let refd_place = self.lower_place(refd);
                let stmt = LIRStatement::Store { dest: target, value: LIRValue { typ: Type::Reference(Box::new(refd_place.typ.clone())), value: LIRValueKind::Reference(refd_place)}}; 
                vec![stmt]
            }
            MIRValueKind::Dereference(reference) => {
                let ref_vreg = self.curr_cell_vreg_map[&reference].clone();
                let ref_type = self.curr_cells[&reference].typ.clone();
                let Type::Reference(deref_typ) = ref_type else {unreachable!()};
                let stmt = LIRStatement::Store { dest: target, value: LIRValue { typ: *deref_typ, value: LIRValueKind::Dereference(ref_vreg)}};
                vec![stmt]
            }
        }
    }

    fn lower_place(&self, place: MIRPlace) -> LIRPlace {
        // TODO: weird solution, change it

        match place.base {
            MIRPlaceBase::Cell(c_id) => {
                let base_type = self.curr_cells[&c_id].typ.clone();
                let (final_offset, final_type) = self.lower_fieldchain(base_type, place.fieldchain);
                LIRPlace {
                    typ: place.typ, 
                    place: LIRPlaceKind::Local{
                        base: self.curr_cell_vreg_map[&c_id], 
                        offset: final_offset 
                    }
                }
            },
            MIRPlaceBase::Deref(c_id) => {
                let ref_type = self.curr_cells[&c_id].typ.clone();
                let Type::Reference(deref_type) = ref_type else {unreachable!()};
                let (final_offset, final_type) = self.lower_fieldchain(*deref_type, place.fieldchain);
                LIRPlace {
                    typ: place.typ,
                    place: LIRPlaceKind::Deref { 
                        pointer: self.curr_cell_vreg_map[&c_id], 
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
        self.curr_cells.insert(id.clone(), cell.clone());
        let cell_vreg_info = VRegInfo { 
            size: self.layouts.get_layout(cell.typ).size(),
            align: 8
        };
        let vreg_id = self.add_vreg(cell_vreg_info);
        self.curr_cell_vreg_map.insert(id, vreg_id);
    }

    fn add_vreg(&mut self, info: VRegInfo) -> VRegId {
        let vreg_id = VRegId(self.vreg_counter);
        self.vreg_counter = self.vreg_counter + 1;
        self.curr_vreg_table.insert(vreg_id.clone(), info);
        vreg_id
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

