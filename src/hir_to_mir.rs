use std::collections::HashMap;

use crate::hir::*;
use crate::hir_to_lir;
use crate::mir::Cell;
use crate::mir::*;
use crate::lir::*;
use crate::shared::typing::*;

pub struct LIRBuilder {
    var_map: HashMap<VarId, CellId>,
    current_cells: HashMap<CellId, Cell>,
    cell_counter: usize,
    block_counter: usize,
    loop_start_stack: Vec<BlockId>,
    loop_end_stack: Vec<BlockId>,
    curr_collected_blocks: HashMap<BlockId, LIRBlock>,
    wip_block_stmts: Vec<LIRStatement>, 
    wip_block_id: Option<BlockId>, 
    typetable: TypeTable,
}


impl LIRBuilder {
    
    pub fn lower_hir(program: HIRProgram) -> LIRProgram {
        let HIRProgram { typetable, functions, entry } = program;
        let mut builder = LIRBuilder {
            var_map: HashMap::new(),
            current_cells: HashMap::new(),
            cell_counter: 0,
            block_counter: 0,
            loop_start_stack: Vec::new(),
            loop_end_stack: Vec::new(),
            curr_collected_blocks: HashMap::new(),
            wip_block_stmts: Vec::new(),
            wip_block_id: None, 
        };
        LIRProgram {
            functions: functions
                .into_iter()
                .map(|(id, func)| (id, builder.lower_function(func)))
                .collect(),
            entry: entry
        }
    }

    fn lower_function(&mut self, func: HIRFunction) -> LIRFunction {
        self.curr_vreg_coll = HashMap::new();
        self.curr_collected_blocks = HashMap::new();
        for (var_id, var) in func.variables.into_iter() {
            let varsize = self.layouts.get_layout(var.typ).size();
            let var_vreg_id = self.add_vreg(varsize);
            self.variable_map.insert(var_id, var_vreg_id.clone());
        }
        let entry_id = self.get_new_blockid();
        self.wip_block_id = Some(entry_id.clone());
        for stmt in func.body.into_iter() {
            self.lower_statement(stmt);
        }
        if !self.wip_block_stmts.is_empty() || self.wip_block_id.is_some() {
            if self.wip_block_id.is_none() {
                self.wip_block_id = Some(self.get_new_blockid());
            }
            self.push_current_block(LIRTerminator::Return(None));
        }
        let arg_ids = func.args
            .iter()
            .map(|arg_id| self.variable_map[&arg_id].clone())
            .collect();
        LIRFunction { 
            blocks: self.curr_collected_blocks.clone(), 
            entry: entry_id, 
            vregs: self.curr_vreg_coll.clone(), 
            args: arg_ids 
        }
    }
    
    fn lower_statement_block(&mut self, stmt_block: Vec<HIRStatement>, entry_id: BlockId, post_jump: BlockId) {
        self.wip_block_id = Some(entry_id);

        for stmt in stmt_block.into_iter() {
            self.lower_statement(stmt);
        }

        // Inserting the tail statements after the last terminator
        if !self.wip_block_stmts.is_empty() || self.wip_block_id.is_some() {
            if self.wip_block_id.is_none() {
                self.wip_block_id = Some(self.get_new_blockid());
            }
            self.push_current_block(LIRTerminator::Goto { dest: post_jump});
        }
    }

    fn push_current_block(&mut self, term: LIRTerminator) {
        let block = LIRBlock {
            statements: self.wip_block_stmts.clone(),
            terminator: term,
        };
        self.wip_block_stmts = Vec::new();
        let block_id = self.wip_block_id.clone().unwrap();
        self.wip_block_id = None;
        self.curr_collected_blocks.insert(block_id, block);
    }

    fn lower_statement(&mut self, stmt: HIRStatement) { 
        match stmt {
            HIRStatement::Let { var, value } => {
                let var_reg = self.variable_map[&var].clone();
                let value_stmts = self.lower_expr_into_target(value, LIRPlace::VReg(var_reg));
                self.wip_block_stmts.extend(value_stmts.into_iter());
            }
            HIRStatement::Assign { target, value } => {
                let lir_target = self.lower_place(target);
                let value_stmts = self.lower_expr_into_target(value, lir_target);
                self.wip_block_stmts.extend(value_stmts.into_iter());
            }
            HIRStatement::If { condition, if_body, else_body } => {
                let branch_id = self.get_new_blockid();
                self.push_current_block(LIRTerminator::Goto { dest: branch_id.clone() });

                let merge_id = self.get_new_blockid();
                let then_id = self.get_new_blockid();
                self.lower_statement_block(if_body, then_id.clone(), merge_id.clone());
                let else_id = match else_body {
                    Some(stmts) => {
                        let id = self.get_new_blockid();
                        self.lower_statement_block(stmts, id.clone(), merge_id.clone());
                        id
                    },
                    None => merge_id.clone()
                };
                let cond_reg = self.add_vreg(8);
                let cond_stmts = self.lower_expr_into_target(condition, LIRPlace::VReg(cond_reg.clone()));
                let branch_block = LIRBlock {
                    statements: cond_stmts,
                    terminator: LIRTerminator::Branch { 
                        condition: Operand::Register(cond_reg), 
                        then_block: then_id, 
                        else_block: else_id,
                    },
                };
                self.curr_collected_blocks.insert(branch_id, branch_block);
                self.wip_block_id = Some(merge_id);
            }
            HIRStatement::While { condition, body }  => {
                let start_id = self.get_new_blockid();
                let body_id = self.get_new_blockid();
                let end_id = self.get_new_blockid();

                self.push_current_block(LIRTerminator::Goto { dest: start_id.clone() });

                let cond_reg = self.add_vreg(8);
                let cond_stmts = self.lower_expr_into_target(condition, LIRPlace::VReg(cond_reg.clone()));
                let loop_header_block = LIRBlock {
                    statements: cond_stmts,
                    terminator: LIRTerminator::Branch { 
                        condition: Operand::Register(cond_reg), 
                        then_block: body_id.clone(), 
                        else_block: end_id.clone() 
                    }
                };
                self.curr_collected_blocks.insert(start_id.clone(), loop_header_block);

                self.loop_start_stack.push(start_id.clone());
                self.loop_end_stack.push(end_id.clone());

                self.lower_statement_block(body, body_id, start_id.clone());
                
                self.wip_block_id = Some(end_id);

                self.loop_start_stack.pop();
                self.loop_end_stack.pop();
            }
            HIRStatement::Break => {
                let break_terminator = LIRTerminator::Goto { dest: self.loop_end_stack.last().unwrap().clone()}; 
                self.push_current_block(break_terminator);
            }
            HIRStatement::Continue => {
                let cont_terminator = LIRTerminator::Goto { dest: self.loop_start_stack.last().unwrap().clone() };
                self.push_current_block(cont_terminator);
            }
            HIRStatement::Return(retval_expr) => {

                // TODO: careful later with larger types !!
                let retval_reg = self.add_vreg(
                    self.layouts.get_layout(retval_expr.typ.clone()).size()
                );

                let retval_stmts = self.lower_expr_into_target(retval_expr, LIRPlace::VReg(retval_reg.clone()));
                self.wip_block_stmts.extend(retval_stmts.into_iter());
                let ret_terminator = LIRTerminator::Return(Some(Operand::Register(retval_reg)));
                self.push_current_block(ret_terminator);
            }
            HIRStatement::Print(expr) => {
                
                let expr_reg = self.add_vreg(
                    self.layouts.get_layout(expr.typ.clone()).size()
                );

                let expr_stmts = self.lower_expr_into_target(expr, LIRPlace::VReg(expr_reg.clone()));
                self.wip_block_stmts.extend(expr_stmts.into_iter());
                let print_stmt = LIRStatement::Print(Operand::Register(expr_reg));
                self.wip_block_stmts.push(print_stmt);
            }
        }
    }

    //-----------------------
     
    fn lower_stmt(&mut self, stmt: HIRStatement) {
        match stmt {
            HIRStatement::Let { var, value } => {
                let cell_id = &self.var_map[&var];
                let target = MIRPlace {
                    typ: self.current_cells[&cell_id].typ,
                    base: cell_id.clone(),
                    fieldchain: Vec::new()
                };
                let stmt = MIRStatement::Assign { target, value: self.lower_expr(value)};
                self.wip_block_stmts.push(stmt);
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
        }
        unimplemented!();
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
            HIRExpressionKind::FieldAccess { expr, field } => { 
                let typ = expr.typ.clone();
                let (expr_val, expr_stmts) = self.lower_expr(*expr);
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
                (MIRValue::StructLiteral{fields: mir_fields}, stmts)
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
