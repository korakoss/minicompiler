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
}

impl LIRBuilder {
    
    pub fn lower_mir(program: MIRProgram) -> LIRProgram {
        let layouts = LayoutTable::make(program.typetable);
        let mut builder = LIRBuilder {
            curr_cell_vreg_map: HashMap::new(),
            curr_cells: HashMap::new(),
            layouts,
            curr_vreg_table: HashMap::new(),
            vreg_counter: 0
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
                let (arg_opnds, arg_stmt_coll): (Vec<LIRValue>, Vec<Vec<LIRStatement>>) = args
                    .into_iter()
                    .map(|arg| self.lower_value_into_operand(arg))
                    .unzip();
                let lir_call = LIRStatement::Call { 
                    dest: lir_target, 
                    func, 
                    args: arg_opnds 
                };
                [arg_stmt_coll.into_iter().flatten().collect(), vec![lir_call]].concat()
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
        match value {
            MIRValue::Place(val_place) => {
                let lir_val_place = self.lower_place(val_place);
                (LIRValue::Place(lir_val_place), Vec::new())
            },
            MIRValue::IntLiteral(num) => {
                (LIRValue::IntLiteral(num), Vec::new())
            },
            MIRValue::BoolTrue => {
                (LIRValue::BoolTrue, Vec::new())
            }
            MIRValue::BoolFalse => {
                (LIRValue::BoolFalse, Vec::new())
            }
            MIRValue::StructLiteral { typ, fields } => {
                let temp_vreg_info = VRegInfo {
                    size: self.layouts.get_layout(typ.clone()).size(),
                    align: 8
                };
                let temp_id = self.add_vreg(temp_vreg_info);
                let temp_place = LIRPlace { base: temp_id, offset: 0};

                // Mehh. Maybe add type info back to MIRV?
                let stmts = self.lower_value_into_place(MIRValue::StructLiteral{typ, fields}, temp_place.clone());
                (LIRValue::Place(temp_place), stmts)
            }
        }
    }

    fn lower_value_into_place(&self, value: MIRValue, target: LIRPlace) -> Vec<LIRStatement> {
        match value {
            MIRValue::Place(val_place) => {
                let lir_val_place = self.lower_place(val_place);
                vec![LIRStatement::Store{dest: target, value: LIRValue::Place(lir_val_place)}] 
            },
            MIRValue::IntLiteral(num) => {
                vec![LIRStatement::Store{dest: target, value: LIRValue::IntLiteral(num)}]
            },
            MIRValue::BoolTrue => {
                vec![LIRStatement::Store{dest: target, value: LIRValue::BoolTrue}]
            }
            MIRValue::BoolFalse => {
                vec![LIRStatement::Store{dest: target, value: LIRValue::BoolFalse}]
            }
            MIRValue::StructLiteral { typ, fields } => {
                let LayoutInfo::Struct { size, field_offsets } = self.layouts.get_layout(typ) else {
                    unreachable!();
                };
                let mut stmts: Vec<LIRStatement> = Vec::new();
                for (fname, fexpr) in fields {
                    let f_target = LIRPlace {
                        base: target.base.clone(),
                        offset: target.offset + field_offsets[&fname]
                    };
                    stmts.extend(self.lower_value_into_place(fexpr, f_target));
                }
                stmts
            }
        }
    }

    fn lower_place(&self, place: MIRPlace) -> LIRPlace {
        // TODO: weird solution, change it

        let base = self.curr_cell_vreg_map[&place.base].clone();

        let mut curr_typ = self.curr_cells[&place.base].typ.clone();
        let mut curr_offs = 0;
        
        for field in place.fieldchain {
            let curr_typ_layout = self.layouts.get_layout(curr_typ.clone());

            match curr_typ_layout {
                LayoutInfo::Struct { size, field_offsets } => {
                    let Type::Derived(TypeConstructor::Struct { fields }) = curr_typ else {
                        unreachable!();
                    };
                    curr_typ = fields[&field].clone();
                    curr_offs = curr_offs + field_offsets[&field];
                } 
                LayoutInfo::Primitive(..) => {
                    panic!("This is primitive, shouldn't have a field");
                }
            }
        }
        LIRPlace{base, offset: curr_offs}
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
    newtype_layouts: HashMap<DerivType, LayoutInfo>
}

impl LayoutTable {

    pub fn make(typetable: TypeTable) -> LayoutTable {
        let mut table = LayoutTable{newtype_layouts: HashMap::new()};
        for tp_id in typetable.topo_order {
            let tp = typetable.newtype_map[&tp_id].clone();
            table.newtype_layouts.insert(tp.clone(), table.make_newtype_layout(tp));
        }
table
    }   

    pub fn get_layout(&self, typ: Type) -> LayoutInfo {
        match typ {
            Type::Prim(prim_tp) => self.get_primitive_layout(prim_tp),
            Type::Derived(tp_constr) => self.newtype_layouts[&tp_constr].clone(),
        }
    }

    fn get_primitive_layout(&self, prim_tp: PrimitiveType) -> LayoutInfo {
        LayoutInfo::Primitive(8)        // Temporarily so; update later
    }
    
    fn make_newtype_layout(&self, deriv_typ: DerivType) -> LayoutInfo {
        
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

