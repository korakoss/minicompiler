use std::collections::HashMap;

use crate::hir::*;
use crate::lir::*;

pub struct LIRBuilder {
    variable_map: HashMap<VarId, VRegId>,
    curr_vreg_coll: HashMap<VRegId, VRegInfo>,
    vreg_counter: usize,
    block_counter: usize,
    loop_start_stack: Vec<BlockId>,
    loop_end_stack: Vec<BlockId>,
    curr_collected_blocks: HashMap<BlockId, LIRBlock>,
    wip_block_stmts: Vec<LIRStatement>, 
    wip_block_id: Option<BlockId>, 
}

impl LIRBuilder {
    
    pub fn lower_hir(program: HIRProgram) -> LIRProgram {
        let mut builder = LIRBuilder {
            variable_map: HashMap::new(),
            curr_vreg_coll: HashMap::new(),
            vreg_counter: 0,
            block_counter: 0,
            loop_start_stack: Vec::new(),
            loop_end_stack: Vec::new(),
            curr_collected_blocks: HashMap::new(),
            wip_block_stmts: Vec::new(),
            wip_block_id: None, 
        };
        builder.lower_hir_program(program)
    }

    fn lower_hir_program(&mut self, program: HIRProgram) -> LIRProgram {
        LIRProgram {
            functions: program.functions
                .into_iter()
                .map(|(id, func)| (id, self.lower_function(func)))
                .collect(),
            entry: program.entry
        }
    }

    fn lower_function(&mut self, func: HIRFunction) -> LIRFunction {
        self.curr_vreg_coll = HashMap::new();
        self.curr_collected_blocks = HashMap::new();
        for (var_id, var) in func.variables.into_iter() {
            let var_vreg_id = self.add_vreg();
            self.variable_map.insert(var_id, var_vreg_id.clone());
        }
        let entry_id = self.get_new_blockid();
        self.lower_statement_block(func.body, entry_id.clone(), None);
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
    
    fn lower_statement_block(&mut self, stmt_block: Vec<HIRStatement>, entry_id: BlockId, post_jump: Option<BlockId>) {
        self.wip_block_id = Some(entry_id);

        for stmt in stmt_block.into_iter() {
            self.lower_statement(stmt);
        }
        
        // Inserting the tail statements after the last terminator
        if !self.wip_block_stmts.is_empty() {
            println!("stmts: {:?}", self.wip_block_stmts);
            let tail_terminator = match post_jump {
                Some(block_id) => LIRTerminator::Goto { dest: block_id },
                None => LIRTerminator::Return(None)
            };
            self.push_current_block(tail_terminator);
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
        println!("Lowering {:?}", stmt);
        match stmt {
            HIRStatement::Let {var:target, value} | HIRStatement::Assign { target, value } => {
                let Place::Variable(var_id) = target;
                let slot = self.variable_map[&var_id].clone();
                let (value_stmts, value_reg) = self.lower_expression(value);
                self.wip_block_stmts.extend(value_stmts.into_iter());
                let assign_stmt = LIRStatement::Load{ 
                    dest: slot, 
                    from: LIRPlace::VReg(value_reg), 
                };
                self.wip_block_stmts.push(assign_stmt);
            }
            HIRStatement::If { condition, if_body, else_body } => {
                let branch_id = self.get_new_blockid();
                self.push_current_block(LIRTerminator::Goto { dest: branch_id.clone() });

                let merge_id = self.get_new_blockid();
                let then_id = self.get_new_blockid();
                self.lower_statement_block(if_body, then_id.clone(), Some(merge_id.clone()));
                let else_id = match else_body {
                    Some(stmts) => {
                        let id = self.get_new_blockid();
                        self.lower_statement_block(stmts, id.clone(), Some(merge_id.clone()));
                        id
                    },
                    None => merge_id.clone()
                };

                let (cond_stmts, cond_reg) = self.lower_expression(condition);
                let branch_block = LIRBlock {
                    statements: cond_stmts,
                    terminator: LIRTerminator::Branch { 
                        condition: Operand::Register(cond_reg), 
                        then_block: then_id, 
                        else_block: else_id,
                    },
                };
                self.curr_collected_blocks.insert(branch_id, branch_block);
            }
            HIRStatement::While { condition, body }  => {
                let start_id = self.get_new_blockid();
                let body_id = self.get_new_blockid();
                let end_id = self.get_new_blockid();

                self.push_current_block(LIRTerminator::Goto { dest: start_id.clone() });

                let (cond_stmts, cond_reg) = self.lower_expression(condition);
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

                self.lower_statement_block(body, body_id, Some(start_id.clone()));
                
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
                let (retval_stmts, retval_reg) = self.lower_expression(retval_expr);
                self.wip_block_stmts.extend(retval_stmts.into_iter());
                let ret_terminator = LIRTerminator::Return(Some(Operand::Register(retval_reg)));
                self.push_current_block(ret_terminator);
            }
            HIRStatement::Print(expr) => {
                let (expr_stmts, expr_reg) = self.lower_expression(expr);
                self.wip_block_stmts.extend(expr_stmts.into_iter());
                let print_stmt = LIRStatement::Print(Operand::Register(expr_reg));
                self.wip_block_stmts.push(print_stmt);
            }
        }
    }


    
    fn lower_expression(&mut self, expr: HIRExpression) -> (Vec<LIRStatement>, VRegId) {
        // Returns the statements to compute the expr and the vreg where the result is
        
        let result_vreg_id = self.add_vreg();

        let stmts = match expr {
            HIRExpression::IntLiteral(num) => {
                let stmt = LIRStatement::Store { 
                    dest: LIRPlace::VReg(result_vreg_id.clone()), 
                    value: Operand::IntLiteral(num), 
            };
                vec![stmt] 
            },
            HIRExpression::Variable(var_id) => {
                let var_slot = self.variable_map[&var_id].clone();
                let stmt = LIRStatement::Load {
                    dest: result_vreg_id.clone(), 
                    from: LIRPlace::VReg(var_slot)
                }; 
                vec![stmt]
            }
            HIRExpression::BinOp { op, left, right } => {
                let (left_stmts, left_reg) = self.lower_expression(*left);
                let (right_stmts, right_reg) = self.lower_expression(*right);
                let binop_stmt = LIRStatement::BinOp { 
                    dest: LIRPlace::VReg(result_vreg_id.clone()), 
                    op, 
                    left: Operand::Register(left_reg), 
                    right:Operand::Register(right_reg), 
                };
                [left_stmts, right_stmts, vec![binop_stmt]].concat()
            }
            HIRExpression::FuncCall { id, args } => {
                let mut arg_vregs: Vec<VRegId> = Vec::new();
                let mut arg_stmts: Vec<Vec<LIRStatement>> = Vec::new();
                for arg in args {
                    let (s, r) = self.lower_expression(arg);
                    arg_vregs.push(r);
                    arg_stmts.push(s);
                }
                let call_stmt = LIRStatement::Call { 
                    dest: LIRPlace::VReg(result_vreg_id.clone()),                      
                    func: id, 
                    args: arg_vregs
                };
                [arg_stmts.into_iter().flatten().collect(), vec![call_stmt]].concat()
            }
            HIRExpression::BoolTrue => {
                let true_stmt = LIRStatement::Store { 
                    dest: LIRPlace::VReg(result_vreg_id.clone()), 
                    value: Operand::BoolTrue
                };
                vec![true_stmt]
            }
            HIRExpression::BoolFalse => {
                let false_stmt = LIRStatement::Store {
                    dest: LIRPlace::VReg(result_vreg_id.clone()), 
                    value: Operand::BoolFalse
                };
                vec![false_stmt]
            }
            _ => {unimplemented!();}        // Struct stuff 
        };
        (stmts, result_vreg_id)
    }

    fn add_vreg(&mut self) -> VRegId {
        // TODO: later add Vreginfo stuff
        let new_id = VRegId(self.vreg_counter);
        self.vreg_counter = self.vreg_counter + 1;
        self.curr_vreg_coll.insert(new_id.clone(), VRegInfo { size: 8, align: 8});
        new_id
    }
    
    fn get_new_blockid(&mut self) -> BlockId {
        let block_id = BlockId(self.block_counter); 
        self.block_counter = self.block_counter + 1;
        block_id 
    }
}

