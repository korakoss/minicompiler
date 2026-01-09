use std::collections::HashMap;

use crate::stages::common::*;
use crate::stages::hir::*;
use crate::stages::mir::*;
use crate::shared::typing::*;


pub struct MIRBuilder {
    var_map: HashMap<VarId, CellId>,
    current_cells: HashMap<CellId, Cell>,
    cell_counter: usize,
    block_counter: usize,
    loop_start_stack: Vec<BlockId>,
    loop_end_stack: Vec<BlockId>,
    curr_collected_blocks: HashMap<BlockId, MIRBlock>,
    wip_blocks: HashMap<BlockId, Vec<MIRStatement>>,
    processing_stack: Vec<BlockId>,
}


impl MIRBuilder {
    
    pub fn lower_hir(program: HIRProgram) -> MIRProgram {
        let mut builder = MIRBuilder {
            var_map: HashMap::new(),
            current_cells: HashMap::new(),
            cell_counter: 0,
            block_counter: 1,
            loop_start_stack: Vec::new(),
            loop_end_stack: Vec::new(),
            curr_collected_blocks: HashMap::new(),
            wip_blocks: HashMap::new(),
            processing_stack: Vec::new()
        };
        MIRProgram {
            functions: program.functions
                .into_iter()
                .map(|(id, func)| (id, builder.lower_function(func)))
                .collect(),
            entry: program.entry,
            typetable: program.typetable
        }
    }

    fn lower_function(&mut self, func: HIRFunction) -> MIRFunction {
        
        self.curr_collected_blocks = HashMap::new();
        self.current_cells = HashMap::new();

        for (var_id, var) in func.variables.into_iter() {
            let cell_id = self.add_cell(Cell { typ: var.typ, kind: CellKind::Var { name: var.name}});
            self.var_map.insert(var_id, cell_id);
        }
        
        let entry_id = self.lower_stmt_block(func.body, MIRTerminator::Return(None));  // TODO: maybe do this nicer somehow (the return part)
        
        let arg_cells: Vec<CellId> = func.args
            .iter()
            .map(|arg_id| self.var_map[&arg_id].clone())
            .collect();
        MIRFunction {
            name: func.name,
            args: arg_cells,
            cells: self.current_cells.clone(),
            blocks: self.curr_collected_blocks.clone(),
            entry: entry_id,
            ret_type: func.ret_type
        }
    }

    fn lower_stmt_block(&mut self, stmts: Vec<HIRStatement>, tail_termr: MIRTerminator) -> BlockId {
        let entry_id = self.add_new_block();

        let mut curr_top_id = entry_id;
        self.switch_to_block(entry_id);

        let mut has_unterminated = false;

        for stmt in stmts {
            self.switch_to_block(curr_top_id);                 // TODO: this is only for safety, sort it out properly
            match self.lower_stmt(stmt) {
                LoweredStatement::Statements(low_stmts) => {
                    self.push_to_current_block(low_stmts);
                    has_unterminated = true;
                }
                LoweredStatement::Termination(low_stmts, term) => {
                    self.push_to_current_block(low_stmts);
                    self.terminate_current_block(term);
                    return entry_id;
                }
                LoweredStatement::TabulaRasa(next_id) => {
                    curr_top_id = next_id;
                    has_unterminated = false;
                }
            }
        }
        
        if has_unterminated {
            self.switch_to_block(curr_top_id);
            self.terminate_current_block(tail_termr);
        }
        entry_id
    }

    
     
    fn lower_stmt(&mut self, stmt: HIRStatement) -> LoweredStatement {
        match stmt {
            HIRStatement::Let { var, value } => {
                let cell_id = &self.var_map[&var];
                let target = MIRPlace {
                    typ: self.current_cells[&cell_id].typ.clone(),
                    base: *cell_id,
                    fieldchain: Vec::new()
                };
                let (mir_val, val_stmts) = self.lower_expr(value);
                LoweredStatement::Statements([val_stmts, vec![MIRStatement::Assign { target, value: mir_val}]].concat())
            },
            HIRStatement::Assign { target, value } => {
                let (mir_val, val_stmts) = self.lower_expr(value);
                LoweredStatement::Statements([val_stmts, vec![MIRStatement::Assign { target: self.lower_place(target), value: mir_val}]].concat())
            }
            HIRStatement::If { condition, if_body, else_body } => {
                let (cond_val, cond_stmts) = self.lower_expr(condition);
                self.push_to_current_block(cond_stmts);
                let curr_id = self.get_current_wip_id().unwrap();

                let merge_id = self.add_new_block();
                let then_id = self.lower_stmt_block(if_body, MIRTerminator::Goto(merge_id));   // This is where it happens
                let else_id = match else_body {
                    Some(else_stmts) => self.lower_stmt_block(else_stmts, MIRTerminator::Goto(merge_id)),
                    None => merge_id,
                };

                self.switch_to_block(curr_id);
                self.terminate_current_block(MIRTerminator::Branch { condition: cond_val, then_: then_id, else_: else_id});
                LoweredStatement::TabulaRasa(merge_id)
            }
            HIRStatement::While { condition, body } => {
                let head_id = self.add_new_block();
                self.loop_start_stack.push(head_id);
                self.terminate_current_block(MIRTerminator::Goto(head_id));
               
                let body_id = self.lower_stmt_block(body, MIRTerminator::Goto(head_id));

                let after_id = self.add_new_block();
                self.loop_end_stack.push(after_id);

                self.switch_to_block(head_id);
                let (cond_val, cond_stmts) = self.lower_expr(condition);
                self.push_to_current_block(cond_stmts);
                self.terminate_current_block(MIRTerminator::Branch { condition: cond_val, then_: body_id, else_: after_id});
                
                self.loop_start_stack.pop();
                self.loop_end_stack.pop();
                LoweredStatement::TabulaRasa(after_id)
            },
            HIRStatement::Break => {
                // TODO: add a "WIP block processing stack", jump back in it here
                LoweredStatement::Termination(vec![], MIRTerminator::Goto(*self.loop_end_stack.last().unwrap())) 
            }
            HIRStatement::Continue => {
                LoweredStatement::Termination(vec![], MIRTerminator::Goto(*self.loop_start_stack.last().unwrap())) 
            }
            HIRStatement::Return(ret_val) => { 
                let (mir_retval, ret_stmts) = match ret_val {
                    Some(value) => {
                        let (mir_val, val_stmts) = self.lower_expr(value);
                        (Some(mir_val), val_stmts)
                    },
                    None => (None, vec![])
                };
                LoweredStatement::Termination(ret_stmts, MIRTerminator::Return(mir_retval))
            }
            HIRStatement::Print(expr) => {
                let (expr_val, expr_stmts) = self.lower_expr(expr);
                LoweredStatement::Statements([expr_stmts, vec![MIRStatement::Print(expr_val)]].concat())
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
            HIRExpressionKind::IntLiteral(num) => {
                (MIRValue{
                    typ: Type::Prim(PrimType::Integer),
                    value: MIRValueKind::IntLiteral(num)
                }, Vec::new())
            },
            HIRExpressionKind::Variable(var_id) => {
                let var_val = MIRValue {
                    typ: expr.typ.clone(),
                    value: MIRValueKind::Place(MIRPlace { 
                        typ: expr.typ, 
                        base: self.var_map[&var_id].clone(),
                        fieldchain: Vec::new(),
                    }),
                };
                (var_val, Vec::new())
            },
            HIRExpressionKind::BinOp { op, left, right } => {
                let (l_val, l_stmts) = self.lower_expr(*left);
                let (r_val, r_stmts) = self.lower_expr(*right);
                let resc_id = self.add_cell(Cell{typ: expr.typ.clone(), kind: CellKind::Temp});
                let target = MIRPlace { typ: expr.typ.clone(), base: resc_id, fieldchain: Vec::new()}; 
                let bin_stmt = MIRStatement::BinOp { 
                    target: target.clone(),
                    op, 
                    left: l_val, 
                    right: r_val 
                };
                (MIRValue{typ: expr.typ, value: MIRValueKind::Place(target)}, [l_stmts, r_stmts, vec![bin_stmt]].concat()) 
            },
            HIRExpressionKind::FuncCall { id, args } => {
                let (arg_vals, arg_stmt_coll): (Vec<MIRValue>, Vec<Vec<MIRStatement>>) = args
                    .into_iter()
                    .map(|arg| self.lower_expr(arg))
                    .unzip();
                let resc_id = self.add_cell(Cell{typ: expr.typ.clone(), kind: CellKind::Temp});
                let target = MIRPlace { typ: expr.typ.clone(), base: resc_id, fieldchain: Vec::new()}; 
                let call_stmt = MIRStatement::Call { target: target.clone(), func: id, args: arg_vals};
                (MIRValue{typ: expr.typ, value: MIRValueKind::Place(target)}, [arg_stmt_coll.into_iter().flatten().collect(), vec![call_stmt]].concat())
            },
            HIRExpressionKind::BoolTrue => (MIRValue{typ: Type::Prim(PrimType::Bool) ,value: MIRValueKind::BoolTrue}, Vec::new()),
            HIRExpressionKind::BoolFalse=> (MIRValue{typ: Type::Prim(PrimType::Bool) ,value: MIRValueKind::BoolFalse}, Vec::new()),
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
                (MIRValue{typ: expr.typ.clone(), value: MIRValueKind::StructLiteral{ typ: expr.typ,fields: mir_fields}}, stmts)
            },
        }
    }

    fn lower_field_access(&self, of: MIRValue, field: String, typ: Type) -> MIRValue {
        let MIRValueKind::Place(place) = of.value else {
           unreachable!(); 
        };
        let mut fieldchain = place.fieldchain;
        fieldchain.push(field);
        let access_place = MIRPlace {
            typ: typ.clone(),
            base: place.base,
            fieldchain 
        };
        MIRValue{typ: typ, value: MIRValueKind::Place(access_place)}
    }
    
    fn push_to_current_block(&mut self, stmts: Vec<MIRStatement>) {
        let curr_id = self.get_current_wip_id().unwrap();
        self.wip_blocks.get_mut(&curr_id).unwrap().extend(stmts.into_iter());
    }

    fn add_new_block(&mut self) -> BlockId {
        let new_id = self.get_new_blockid();
        self.wip_blocks.insert(new_id, Vec::new());
        new_id
    }

    fn switch_to_block(&mut self, id: BlockId) {
        if !self.wip_blocks.contains_key(&id) {
            panic!("Block with requested ID {:?} not found. Current WIP blocks: {:?}", id, self.wip_blocks);
        }
        self.processing_stack.push(id);
    }

    fn terminate_current_block(&mut self, terminator: MIRTerminator) {
        let curr_id = self.get_current_wip_id().unwrap();
        let statements = self.wip_blocks.remove(&curr_id).unwrap();
        let block = MIRBlock {
            statements,
            terminator,
        };
        self.curr_collected_blocks.insert(curr_id, block);
        self.processing_stack.pop();
    }

    fn get_current_wip_id(&mut self) -> Option<BlockId> {
        self.processing_stack.last().copied()
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


enum LoweredStatement {
    Statements(Vec<MIRStatement>),
    Termination(Vec<MIRStatement>, MIRTerminator),
    TabulaRasa(BlockId),
}
