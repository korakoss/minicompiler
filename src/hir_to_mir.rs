use std::collections::HashMap;

use crate::hir::*;
use crate::mir::*;
use crate::lir::*;
use crate::shared::typing::*;


pub struct MIRBuilder {
    var_map: HashMap<VarId, CellId>,
    current_cells: HashMap<CellId, Cell>,
    cell_counter: usize,
    block_counter: usize,
    loop_start_stack: Vec<BlockId>,
    loop_end_stack: Vec<BlockId>,
    curr_collected_blocks: HashMap<BlockId, MIRBlock>,
    wip_block_stmts: Vec<MIRStatement>, 
    wip_block_id: BlockId, 
    typetable: TypeTable,
}


impl MIRBuilder {
    
    pub fn lower_hir(program: HIRProgram) -> MIRProgram {
        let HIRProgram { typetable, functions, entry } = program;
        let mut builder = MIRBuilder {
            var_map: HashMap::new(),
            current_cells: HashMap::new(),
            cell_counter: 0,
            block_counter: 1,
            loop_start_stack: Vec::new(),
            loop_end_stack: Vec::new(),
            curr_collected_blocks: HashMap::new(),
            wip_block_stmts: Vec::new(),
            wip_block_id: BlockId(0), 
            typetable: typetable,
        };
        MIRProgram {
            functions: functions
                .into_iter()
                .map(|(id, func)| (id, builder.lower_function(func)))
                .collect(),
            entry: entry,
            typetable: builder.typetable
        }
    }

    fn lower_function(&mut self, func: HIRFunction) -> MIRFunction {
        
        self.curr_collected_blocks = HashMap::new();

        self.current_cells = HashMap::new();
        for (var_id, var) in func.variables.into_iter() {
            let cell_id = self.add_cell(Cell { typ: var.typ, kind: CellKind::Var { name: var.name}});
            self.var_map.insert(var_id, cell_id);
        }
        
        let entry_id = self.wip_block_id.clone();
        self.lower_stmt_block(func.body, MIRTerminator::Return(None));  // TODO: maybe do this nicer somehow
        
        let arg_cells: Vec<CellId> = func.args
            .iter()
            .map(|arg_id| self.var_map[&arg_id].clone())
            .collect();
        let func = MIRFunction {
            name: func.name,
            args: arg_cells,
            cells: self.current_cells.clone(),
            blocks: self.curr_collected_blocks.clone(),
            entry: entry_id,
            ret_type: func.ret_type
        };
        self.current_cells = HashMap::new();
        self.curr_collected_blocks = HashMap::new();
        func
    }
    
    fn push_wip(&mut self, terminator: MIRTerminator) {
        let id = self.get_wip_id();
        let statements = self.get_wip_stmts();
        let block = MIRBlock {statements, terminator};
        self.curr_collected_blocks.insert(id, block);
    }

    fn get_wip_id(&mut self) -> BlockId {
        let id = self.wip_block_id.clone();
        self.wip_block_id = self.get_new_blockid();
        id
    }

    fn get_wip_stmts(&mut self) -> Vec<MIRStatement> {
        let stmts = self.wip_block_stmts.clone();
        self.wip_block_stmts = Vec::new();
        stmts
    }


    fn lower_stmt_block(&mut self, stmts: Vec<HIRStatement>, tail_termr: MIRTerminator) -> BlockId {
        let entry_id = self.wip_block_id.clone();

        for stmt in stmts.into_iter() {
            self.lower_stmt(stmt);
        }

        // MEH! TODO: think abt this
        self.push_wip(tail_termr);
        entry_id
    }
     
    fn lower_stmt(&mut self, stmt: HIRStatement) {

        match stmt {
            HIRStatement::Let { var, value } => {
                let cell_id = &self.var_map[&var];
                let target = MIRPlace {
                    typ: self.current_cells[&cell_id].typ.clone(),
                    base: cell_id.clone(),
                    fieldchain: Vec::new()
                };
                let (mir_val, val_stmts) = self.lower_expr(value);
                self.wip_block_stmts.extend(val_stmts.into_iter());
                self.wip_block_stmts.push(MIRStatement::Assign { target, value: mir_val});
            },
            HIRStatement::Assign { target, value } => {
                let (mir_val, val_stmts) = self.lower_expr(value);
                let mir_assign = MIRStatement::Assign { 
                    target: self.lower_place(target), 
                    value: mir_val,
                };
                self.wip_block_stmts.extend(val_stmts.into_iter());
                self.wip_block_stmts.push(mir_assign);
            }
            HIRStatement::If { condition, if_body, else_body } => {
                let (cond_val, cond_stmts) = self.lower_expr(condition);
                self.wip_block_stmts.extend(cond_stmts.into_iter());
                let curr_stmts = self.get_wip_stmts();
                let curr_id = self.get_wip_id();

                let merge_id = self.get_new_blockid();
                let then_block_id = self.lower_stmt_block(if_body, MIRTerminator::Goto(merge_id.clone()));
                let else_block_id = match else_body {
                    Some(else_stmts) => self.lower_stmt_block(else_stmts, MIRTerminator::Goto(merge_id.clone())),
                    None => merge_id.clone()
                };
                let block = MIRBlock{statements: curr_stmts, terminator: MIRTerminator::Branch { condition: cond_val, then_: then_block_id, else_: else_block_id}};
                self.curr_collected_blocks.insert(curr_id, block);
                self.wip_block_id = merge_id;
            }
            HIRStatement::While { condition, body } => {
                let head_id = self.get_new_blockid();
                self.push_wip(MIRTerminator::Goto(head_id.clone()));
                
                let (cond_val, cond_stmts) = self.lower_expr(condition);
                let body_id = self.lower_stmt_block(body, MIRTerminator::Goto(head_id.clone()));
                let after_id = self.wip_block_id.clone();
                let head_block = MIRBlock {
                    statements: cond_stmts,
                    terminator: MIRTerminator::Branch { 
                        condition: cond_val, 
                        then_: body_id, 
                        else_: after_id
                    }
                };
                self.curr_collected_blocks.insert(head_id, head_block);
            },
            HIRStatement::Break => {
                let break_terminator = MIRTerminator::Goto(self.loop_end_stack.last().unwrap().clone()); 
                self.push_wip(break_terminator);
            }
            HIRStatement::Continue => {
                let cont_terminator = MIRTerminator::Goto(self.loop_start_stack.last().unwrap().clone()); 
                self.push_wip(cont_terminator);
            }
            HIRStatement::Return(ret_val) => { 
                let (mir_retval, ret_stmts) = self.lower_expr(ret_val);
                self.wip_block_stmts.extend(ret_stmts.into_iter());
                let ret_terminator = MIRTerminator::Return(Some(mir_retval));
                self.push_wip(ret_terminator);
            }
            HIRStatement::Print(expr) => {
                let (expr_val, expr_stmts) = self.lower_expr(expr);
                self.wip_block_stmts.extend(expr_stmts.into_iter());
                self.wip_block_stmts.push(MIRStatement::Print(expr_val));
            }
        }
    }
    

    fn lower_place(&self, hir_place: Place) -> MIRPlace {
       match hir_place.place {
            PlaceKind::Variable(var_id) => MIRPlace { 
               typ: hir_place.typ, 
               base: self.var_map[&var_id].clone(), 
               fieldchain: Vec::new(),
            },
            PlaceKind::StructField { of, field } => {
                let MIRPlace { typ, base, fieldchain } = self.lower_place(*of);
                MIRPlace {
                    typ: hir_place.typ,
                    base,
                    fieldchain: [fieldchain, vec![field]].concat(),
                }
            }
        }
    }

    fn lower_expr(&mut self, expr: HIRExpression) -> (MIRValue, Vec<MIRStatement>) {
        match expr.expr {
            HIRExpressionKind::IntLiteral(num) => (MIRValue::IntLiteral(num), Vec::new()),
            HIRExpressionKind::Variable(var_id) => {
                let var_val = MIRValue::Place(MIRPlace { 
                    typ: expr.typ.clone(), 
                    base: self.var_map[&var_id].clone(),
                    fieldchain: Vec::new(),
                });
                (var_val, Vec::new())
            },
            HIRExpressionKind::BinOp { op, left, right } => {
                let (l_val, l_stmts) = self.lower_expr(*left);
                let (r_val, r_stmts) = self.lower_expr(*right);
                let resc_id = self.add_cell(Cell{typ: expr.typ.clone(), kind: CellKind::Temp});
                let target = MIRPlace { typ: expr.typ, base: resc_id, fieldchain: Vec::new()}; 
                let bin_stmt = MIRStatement::BinOp { 
                    target: target.clone(),
                    op, 
                    left: l_val, 
                    right: r_val 
                };
                (MIRValue::Place(target), [l_stmts, r_stmts, vec![bin_stmt]].concat()) 
            },
            HIRExpressionKind::FuncCall { id, args } => {
                let (arg_vals, arg_stmt_coll): (Vec<MIRValue>, Vec<Vec<MIRStatement>>) = args
                    .into_iter()
                    .map(|arg| self.lower_expr(arg))
                    .unzip();
                let resc_id = self.add_cell(Cell{typ: expr.typ.clone(), kind: CellKind::Temp});
                let target = MIRPlace { typ: expr.typ, base: resc_id, fieldchain: Vec::new()}; 
                let call_stmt = MIRStatement::Call { target: target.clone(), func: id, args: arg_vals};
                (MIRValue::Place(target), [arg_stmt_coll.into_iter().flatten().collect(), vec![call_stmt]].concat())
            },
            HIRExpressionKind::BoolTrue => (MIRValue::BoolTrue, Vec::new()),
            HIRExpressionKind::BoolFalse => (MIRValue::BoolFalse, Vec::new()),
            HIRExpressionKind::FieldAccess { expr: base_expr, field } => { 
                let typ = expr.typ.clone();
                let (expr_val, expr_stmts) = self.lower_expr(*base_expr);
                let access_val = self.lower_field_access(expr_val, field, typ);
                (access_val, expr_stmts)
            },
            HIRExpressionKind::StructLiteral {fields} => {
                let mut stmts: Vec<MIRStatement> = Vec::new();
                let mut mir_fields: HashMap<String, MIRValue> = HashMap::new();
                for (fname, fexpr) in fields {
                    let (f_val, f_stmts) = self.lower_expr(fexpr);
                    stmts.extend(f_stmts.into_iter());
                    mir_fields.insert(fname, f_val);
                }
                (MIRValue::StructLiteral{ typ: expr.typ,fields: mir_fields}, stmts)
            },
        }
    }

    fn lower_field_access(&self, of: MIRValue, field: String, typ: Type) -> MIRValue {
        let MIRValue::Place(place) = of else {
           unreachable!(); 
        };
        let mut fieldchain = place.fieldchain;
        fieldchain.push(field);
        let access_place = MIRPlace {
            typ,
            base: place.base,
            fieldchain 
        };
        MIRValue::Place(access_place)
    }
        
    fn get_new_blockid(&mut self) -> BlockId {
        let block_id = BlockId(self.block_counter); 
        self.block_counter = self.block_counter + 1;
        block_id 
    }

    fn add_cell(&mut self, cell: Cell) -> CellId {
        let new_id = CellId(self.cell_counter);
        self.cell_counter = self.cell_counter + 1;
        self.current_cells.insert(new_id.clone(), cell);
        new_id
    }
}
