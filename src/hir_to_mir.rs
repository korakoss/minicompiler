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
                    base: MIRPlaceBase::Cell(*cell_id),
                    fieldchain: Vec::new()
                };
                let (mir_val, val_stmts) = self.lower_expr(value);
                LoweredStatement::Statements([val_stmts, vec![MIRStatement::Assign { target, value: mir_val}]].concat())
            },
            HIRStatement::Assign { target, value } => {
                let (mir_val, val_stmts) = self.lower_expr(value);
                let (mir_target, target_stmts) =  self.lower_place(target);
                LoweredStatement::Statements([val_stmts, target_stmts, vec![MIRStatement::Assign { target: mir_target, value: mir_val}]].concat())
            }
            HIRStatement::If { condition, if_body, else_body } => {
                let (cond_val, cond_stmts) = self.lower_expr(condition);
                self.push_to_current_block(cond_stmts);
                let curr_id = self.get_current_wip_id().unwrap();

                let merge_id = self.add_new_block();
                let then_id = self.lower_stmt_block(if_body, MIRTerminator::Goto(merge_id));   
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
                
                let after_id = self.add_new_block();
                self.loop_end_stack.push(after_id);

                let body_id = self.lower_stmt_block(body, MIRTerminator::Goto(head_id));

                
                self.switch_to_block(head_id);
                let (cond_val, cond_stmts) = self.lower_expr(condition);
                self.push_to_current_block(cond_stmts);
                self.terminate_current_block(MIRTerminator::Branch { condition: cond_val, then_: body_id, else_: after_id});
                
                self.loop_start_stack.pop();
                self.loop_end_stack.pop();
                LoweredStatement::TabulaRasa(after_id)
            },
            HIRStatement::Break => {
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
    

    fn lower_place(&mut self, hir_place: Place) -> (MIRPlace, Vec<MIRStatement>) {
       match hir_place.place {
            PlaceKind::Variable(var_id) => (MIRPlace { 
               typ: hir_place.typ, 
               base: MIRPlaceBase::Cell(self.var_map[&var_id].clone()), 
               fieldchain: Vec::new(),
            }, vec![]),
            PlaceKind::StructField { of, field } => {
                let (MIRPlace {typ: _,base, fieldchain }, of_stmts) = self.lower_place(*of);
                (MIRPlace {
                    typ: hir_place.typ,
                    base,
                    fieldchain: [fieldchain, vec![field]].concat(),
                }, of_stmts)
            }
            PlaceKind::Deref(reference) => {
                let (ref_val, ref_stmts) = self.lower_expr(reference); 

                let ref_val_cell = self.add_cell(Cell { 
                    typ:  ref_val.typ.clone(),
                    kind: CellKind::Temp 
                });
                let ref_assign_stmt = MIRStatement::Assign { 
                    target: MIRPlace { 
                        typ: ref_val.typ.clone(), 
                        base: MIRPlaceBase::Cell(ref_val_cell), 
                        fieldchain: vec![], 
                    }, 
                    value: ref_val
                };
                (MIRPlace {
                    typ: hir_place.typ,
                    base: MIRPlaceBase::Deref(ref_val_cell),
                    fieldchain: vec![]
                }, [ref_stmts, vec![ref_assign_stmt]].concat())
            }
        }
    }

    fn lower_expr(&mut self, expr: HIRExpression) -> (MIRValue, Vec<MIRStatement>) {
        match expr.expr {
            HIRExpressionKind::IntLiteral(num) => {
                (MIRValue{
                    typ: ConcreteType::Prim(PrimType::Integer),
                    value: MIRValueKind::IntLiteral(num)
                }, Vec::new())
            },
            HIRExpressionKind::Variable(var_id) => {
                let var_val = MIRValue {
                    typ: expr.typ.clone(),
                    value: MIRValueKind::Place(MIRPlace { 
                        typ: expr.typ, 
                        base: MIRPlaceBase::Cell(self.var_map[&var_id].clone()),
                        fieldchain: Vec::new(),
                    }),
                };
                (var_val, Vec::new())
            },
            HIRExpressionKind::BinOp { op, left, right } => {
                let (l_val, l_stmts) = self.lower_expr(*left);
                let (r_val, r_stmts) = self.lower_expr(*right);
                let resc_id = self.add_cell(Cell{typ: expr.typ.clone(), kind: CellKind::Temp});
                let target = MIRPlace { 
                    typ: expr.typ.clone(), 
                    base: MIRPlaceBase::Cell(resc_id), 
                    fieldchain: Vec::new()
                }; 
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
                let target = MIRPlace { 
                    typ: expr.typ.clone(), 
                    base: MIRPlaceBase::Cell(resc_id), 
                    fieldchain: Vec::new()
                }; 
                let call_stmt = MIRStatement::Call { target: target.clone(), func: id, args: arg_vals};
                (MIRValue{typ: expr.typ, value: MIRValueKind::Place(target)}, [arg_stmt_coll.into_iter().flatten().collect(), vec![call_stmt]].concat())
            },
            HIRExpressionKind::BoolTrue => (MIRValue{typ: ConcreteType::Prim(PrimType::Bool) ,value: MIRValueKind::BoolTrue}, Vec::new()),
            HIRExpressionKind::BoolFalse=> (MIRValue{typ: ConcreteType::Prim(PrimType::Bool) ,value: MIRValueKind::BoolFalse}, Vec::new()),
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
            HIRExpressionKind::Reference(refd) => {
                let (mir_refd, refd_stmts) = self.lower_expr(*refd);
                match mir_refd.value {
                    MIRValueKind::Place(refd_place) => {
                        (MIRValue{typ: ConcreteType::Reference(Box::new(mir_refd.typ)), value: MIRValueKind::Reference(refd_place)}, refd_stmts)
                    }
                    MIRValueKind::Reference(refd_ref) => {
                        let tempc = self.add_cell(Cell{typ: mir_refd.typ.clone(), kind: CellKind::Temp});
                        let temp_place = MIRPlace{typ: mir_refd.typ.clone(), base: MIRPlaceBase::Cell(tempc), fieldchain: vec![]};
                        let assign_stmt = MIRStatement::Assign { target: temp_place.clone(), value: MIRValue { typ: mir_refd.typ.clone(), value: MIRValueKind::Reference(refd_ref)}};
                        (MIRValue{typ: expr.typ, value: MIRValueKind::Reference(temp_place)}, [refd_stmts, vec![assign_stmt]].concat())
                    }
                    _ => {unreachable!();}
                }
            }

            HIRExpressionKind::Dereference(reference) => {
                let (ref_val, ref_stmts) = self.lower_expr(*reference); 

                let ref_val_cell = self.add_cell(Cell { 
                    typ:  ref_val.typ.clone(),
                    kind: CellKind::Temp
                });
                let ref_assign_stmt = MIRStatement::Assign { 
                    target: MIRPlace { 
                        typ: ref_val.typ.clone(), 
                        base: MIRPlaceBase::Cell(ref_val_cell), 
                        fieldchain: vec![], 
                    }, 
                    value: ref_val
                };
                (MIRValue {
                    typ: expr.typ.clone(), 
                    value: MIRValueKind::Place(MIRPlace { 
                        typ: expr.typ, 
                        base: MIRPlaceBase::Deref(ref_val_cell), 
                        fieldchain: vec![], 
                    }),
                }, [ref_stmts, vec![ref_assign_stmt]].concat())

            }
        }
    }

    fn lower_field_access(&self, of: MIRValue, field: String, typ: ConcreteType) -> MIRValue {
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
