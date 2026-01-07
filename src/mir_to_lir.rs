use std::collections::HashMap;

use crate::hir::*;
use crate::lir::*;
use crate::mir::*;

use crate::shared::typing::*;


pub struct LIRBuilder {
    curr_cell_map: HashMap<CellId, VRegId>,
    layouts: LayoutTable,
    vreg_table: HashMap<VRegId, VRegInfo>,
    vreg_counter: usize,
}

impl LIRBuilder {
    
    pub fn lower_mir(program: MIRProgram) -> LIRProgram {
        let layouts = LayoutTable::make(program.typetable);
        let mut builder = LIRBuilder {
            curr_cell_map: HashMap::new(),
            layouts,
            vreg_table: HashMap::new(),
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
        for (id, cell) in func.cells {
            self.lower_cell(id, cell);
        }
        unimplemented!();
    }

    fn lower_block(&self, block: MIRBlock) -> LIRBlock {
        unimplemented!();
    }

    fn lower_stmt(&self, stmt: MIRStatement) -> Vec<LIRStatement> {
        match stmt {
            MIRStatement::Assign { target, value } => {
                let lir_target = self.lower_place(target);
                self.lower_value_into_place(value, lir_target)
            }
            _ => {unimplemented!();}
        }
    }

    fn lower_terminator(&self, term: MIRTerminator) -> LIRTerminator {
        unimplemented!();
    }

    fn lower_value_into_operand(&mut self, value: MIRValue) -> (Operand, Vec<LIRStatement>) {
        match value {
            MIRValue::Place(val_place) => {
                let lir_val_place = self.lower_place(val_place);
                (Operand::Place(lir_val_place), Vec::new())
            },
            MIRValue::IntLiteral(num) => {
                (Operand::IntLiteral(num), Vec::new())
            },
            MIRValue::BoolTrue => {
                (Operand::BoolTrue, Vec::new())
            }
            MIRValue::BoolFalse => {
                (Operand::BoolFalse, Vec::new())
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
                (Operand::Place(temp_place), stmts)
            }
        }
    }

    fn lower_value_into_place(&self, value: MIRValue, target: LIRPlace) -> Vec<LIRStatement> {
        match value {
            MIRValue::Place(val_place) => {
                let lir_val_place = self.lower_place(val_place);
                vec![LIRStatement::Store{dest: target, value: Operand::Place(lir_val_place)}] 
            },
            MIRValue::IntLiteral(num) => {
                vec![LIRStatement::Store{dest: target, value: Operand::IntLiteral(num)}]
            },
            MIRValue::BoolTrue => {
                vec![LIRStatement::Store{dest: target, value: Operand::BoolTrue}]
            }
            MIRValue::BoolFalse => {
                vec![LIRStatement::Store{dest: target, value: Operand::BoolFalse}]
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
            _ => {unimplemented!();}
        }
    }

    fn lower_place(&self, place: MIRPlace) -> LIRPlace {
        let base = self.curr_cell_map[&place.base].clone();

        let mut curr_typ = place.typ;
        let mut curr_offs = 0;

        for field in place.fieldchain {
            let curr_typ_layout = self.layouts.get_layout(curr_typ.clone());
            let LayoutInfo::Struct { size, field_offsets } = curr_typ_layout else {
                unreachable!();
            };
            let Type::Derived(TypeConstructor::Struct { fields }) = curr_typ else {
                unreachable!();
            };
            curr_typ = fields[&field].clone();
            curr_offs = curr_offs + field_offsets[&field];
        }
        LIRPlace{base, offset: curr_offs}
    }
    
    fn lower_cell(&mut self, id: CellId, cell: Cell) -> VRegId {
        let cell_vreg_info = VRegInfo { 
            size: self.layouts.get_layout(cell.typ).size(),
            align: 8
        };
        let vreg_id = self.add_vreg(cell_vreg_info);
        self.curr_cell_map.insert(id, vreg_id.clone());
    }

    fn add_vreg(&mut self, info: VRegInfo) -> VRegId {
        let vreg_id = VRegId(self.vreg_counter);
        self.vreg_counter = self.vreg_counter + 1;
        self.vreg_table.insert(vreg_id.clone(), cell_vreg_info);
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

        // TODO: we have to process in topo order !!!!!!!
        // Currently, I think it spills down in that order here
        // But we should make it cleaner
        
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

