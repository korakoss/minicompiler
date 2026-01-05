use std::collections::HashMap;

use crate::hir::*;
use crate::lir::*;


struct LIRBuilder {
    blocks: HashMap<BlockId, LIRBlock>,
    func_entries: HashMap<FuncId, BlockId>,
    variable_map: HashMap<VarId, VRegId>,
    vreg_counter: usize,
    block_counter: usize,
    loop_start_stack: Vec<BlockId>,
    loop_end_stack: Vec<BlockId>,
    curr_collected_stmts: Vec<LIRStatement>, 
    block_id_stack: Vec<BlockId>,
}

impl LIRBuilder {

    fn lower_hir_program(&mut self, hir_program: HIRProgram) -> LIRProgram {
        
        // TODO: something with setting up args and vars
        for (func_id, func) in hir_program.functions {
            let body_blockid = self.lower_statement_block(func.body, None);
            self.func_entries.insert(func_id, body_blockid);
        }
        unimplemented!();
    }
    
    fn lower_statement_block(&mut self, stmt_block: Vec<HIRStatement>, post_jump: Option<BlockId>) -> BlockId{
        let entry_id = self.get_new_blockid();
        self.block_id_stack.push(entry_id.clone());

        for stmt in stmt_block.into_iter() {
            self.lower_statement(stmt);
        }
        
        // Inserting the tail statements after the last terminator

        let tail_terminator = match post_jump {
            Some(block_id) => LIRTerminator::Goto { dest: block_id },
            None => LIRTerminator::Return(None)
        };
        self.push_current_block(tail_terminator);
        entry_id
    }

    fn push_current_block(&mut self, term: LIRTerminator) {
        let block = LIRBlock {
            statements: self.curr_collected_stmts.clone(),
            terminator: term,
        };
        let block_id = self.block_id_stack.pop().unwrap();
        self.blocks.insert(block_id, block);
    }

    fn lower_statement(&mut self, stmt: HIRStatement) { 

        match stmt {
            HIRStatement::Let {var:target, value} | HIRStatement::Assign { target, value } => {
                let Place::Variable(var_id) = target;
                let slot = self.variable_map[&var_id].clone();
                let (value_stmts, value_reg) = self.lower_expression(value);
                self.curr_collected_stmts.extend(value_stmts.into_iter());
                let assign_stmt = LIRStatement::Load{ 
                    dest: slot, 
                    from: LIRPlace::VReg(value_reg), 
                };
                self.curr_collected_stmts.push(assign_stmt);
            }
            HIRStatement::If { condition, if_body, else_body } => {
                let (cond_stmts, cond_reg) = self.lower_expression(condition);
                self.curr_collected_stmts.extend(cond_stmts.into_iter());
                let merge_id = self.get_new_blockid();
                let then_block_id = self.lower_statement_block(if_body, Some(merge_id.clone()));
                let else_block_id = match else_body {
                    Some(stmts) => self.lower_statement_block(stmts, Some(merge_id.clone())),
                    None => merge_id.clone()
                };
                let branch_term = LIRTerminator::Branch { 
                    condition: Operand::Register(cond_reg), 
                    then_block: then_block_id, 
                    else_block: else_block_id
                };
                self.push_current_block(branch_term);
                self.block_id_stack.push(merge_id);
            }
            HIRStatement::While { condition, body }  => {
                let (cond_stmts, cond_reg) = self.lower_expression(condition);
                self.curr_collected_stmts.extend(cond_stmts.into_iter());
                let start_id = self.get_new_blockid();
                let end_id = self.get_new_blockid();
                self.loop_start_stack.push(start_id.clone());
                self.loop_end_stack.push(end_id.clone());
                let body_id = self.lower_statement_block(body, Some(start_id.clone()));
                let loop_terminator = LIRTerminator::Branch { 
                    condition: Operand::Register(cond_reg), 
                    then_block: body_id, 
                    else_block: end_id.clone() 
                };
                self.push_current_block(loop_terminator);  
                
                self.block_id_stack.push(end_id);

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
                self.curr_collected_stmts.extend(retval_stmts.into_iter());
                let ret_terminator = LIRTerminator::Return(Some(Operand::Register(retval_reg)));
                self.push_current_block(ret_terminator);
            }
            HIRStatement::Print(expr) => {
                let (expr_stmts, expr_reg) = self.lower_expression(expr);
                self.curr_collected_stmts.extend(expr_stmts.into_iter());
                let print_stmt = LIRStatement::Print(Operand::Register(expr_reg));
                self.curr_collected_stmts.push(print_stmt);
            }
        }
    }


    
    fn lower_expression(&mut self, expr: HIRExpression) -> (Vec<LIRStatement>, VRegId) {
        // Returns the statements to compute the expr and the vreg where the result is
        
        let result_vreg = self.get_new_vreg();

        let stmts = match expr {
            HIRExpression::IntLiteral(num) => {
                let stmt = LIRStatement::Store { 
                    dest: LIRPlace::VReg(result_vreg.clone()), 
                    value: Operand::IntLiteral(num), 
            };
                vec![stmt] 
            },
            HIRExpression::Variable(var_id) => {
                let var_slot = self.variable_map[&var_id].clone();
                let stmt = LIRStatement::Load {
                    dest: result_vreg.clone(), 
                    from: LIRPlace::VReg(var_slot)
                }; 
                vec![stmt]
            }
            HIRExpression::BinOp { op, left, right } => {
                let (left_stmts, left_reg) = self.lower_expression(*left);
                let (right_stmts, right_reg) = self.lower_expression(*right);
                let binop_stmt = LIRStatement::BinOp { 
                    dest: LIRPlace::VReg(result_vreg.clone()), 
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
                    dest: LIRPlace::VReg(result_vreg.clone()),                      
                    func: id, 
                    args: arg_vregs
                };
                [arg_stmts.into_iter().flatten().collect(), vec![call_stmt]].concat()
            }
            HIRExpression::BoolTrue => {
                let true_stmt = LIRStatement::Store { 
                    dest: LIRPlace::VReg(result_vreg.clone()), 
                    value: Operand::BoolTrue
                };
                vec![true_stmt]
            }
            HIRExpression::BoolFalse => {
                let false_stmt = LIRStatement::Store {
                    dest: LIRPlace::VReg(result_vreg.clone()), 
                    value: Operand::BoolFalse
                };
                vec![false_stmt]
            }
            _ => {unimplemented!();}        // Struct stuff 
        };
        (stmts, result_vreg)
    }
    
    fn get_new_vreg(&mut self) -> VRegId {
        let vreg = VRegId(self.vreg_counter);
        self.vreg_counter = self.vreg_counter + 1;
        vreg
    }

    fn get_new_blockid(&mut self) -> BlockId {
        let block_id = BlockId(self.block_counter); 
        self.block_counter = self.block_counter + 1;
        block_id 
    }
}

