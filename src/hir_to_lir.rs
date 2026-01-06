use std::collections::HashMap;

use crate::hir::*;
use crate::lir::*;
use crate::shared::typing::*;


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
    layouts: LayoutTable,
}

impl LIRBuilder {
    
    pub fn lower_hir(program: HIRProgram) -> LIRProgram {
        let HIRProgram { typetable, functions, entry } = program;
        let layouts = LayoutTable::make(typetable);
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
            layouts
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

    fn lower_place(&self, hir_place: Place) -> LIRPlace {
        match hir_place.place {
            PlaceKind::Variable(var_id) => {
                LIRPlace::VReg(self.variable_map[&var_id].clone())
            }
            PlaceKind::StructField { of, field } => {
                let LayoutInfo::Struct { size, field_offsets } = self.layouts.get_layout(of.typ.clone()) else {
                    unreachable!();
                };
                let f_offset = field_offsets[&field];
                match self.lower_place(*of) {
                    LIRPlace::VReg(vreg_id) => LIRPlace::Deref { base: vreg_id, offset: f_offset },
                    LIRPlace::Deref { base, offset } => LIRPlace::Deref { base, offset: offset + f_offset }
                }
            }
        } 
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
    fn lower_expr_into_target(&mut self, expr: HIRExpression, target: LIRPlace) -> Vec<LIRStatement> {
        match expr.expr {
            HIRExpressionKind::StructLiteral { fields } => {
                let LayoutInfo::Struct { field_offsets, .. } = self.layouts.get_layout(expr.typ) else {
                    unreachable!();
                };
                let base = match &target {
                    LIRPlace::VReg(vreg) => vreg.clone(),
                    LIRPlace::Deref { base, .. } => base.clone(),
                };
                let base_offset = match &target {
                    LIRPlace::VReg(_) => 0,
                    LIRPlace::Deref { offset, .. } => *offset,
                };
                let mut stmts = Vec::new();
                for (fname, fexpr) in fields {
                    let f_target = LIRPlace::Deref { 
                        base: base.clone(), 
                        offset: base_offset + field_offsets[&fname] 
                    };
                    stmts.extend(self.lower_expr_into_target(fexpr, f_target));
                }
                stmts
            }
            _ => {
                let (opnd, stmts) = self.lower_expr_into_opnd(expr);
                [stmts, vec![LIRStatement::Store { dest: target, value: opnd }]].concat()
            }
        }
    }
    
    fn lower_expr_into_opnd(&mut self, expr: HIRExpression) -> (Operand, Vec<LIRStatement>) {
        match expr.expr {
            HIRExpressionKind::IntLiteral(num) => (Operand::IntLiteral(num), Vec::new()),
            HIRExpressionKind::Variable(var_id) => {
                let var_reg = self.variable_map[&var_id].clone();
                (Operand::Register(var_reg), Vec::new())
            }
            HIRExpressionKind::BinOp { op, left, right } => {
                let (left_opnd, left_stmts) = self.lower_expr_into_opnd(*left);
                let (right_opnd, right_stmts) = self.lower_expr_into_opnd(*right);
                let res_vreg = self.add_vreg(self.layouts.get_layout(expr.typ.clone()).size());
                let bin_stmt = LIRStatement::BinOp { dest: LIRPlace::VReg(res_vreg.clone()), op, left: left_opnd, right: right_opnd };
                (Operand::Register(res_vreg), [left_stmts, right_stmts, vec![bin_stmt]].concat())
            }
            HIRExpressionKind::FuncCall { id, args } => {
                let (arg_opnds, arg_stmt_coll): (Vec<Operand>, Vec<Vec<LIRStatement>>) = args
                    .into_iter()
                    .map(|arg| self.lower_expr_into_opnd(arg))
                    .unzip();
                let res_vreg = self.add_vreg(self.layouts.get_layout(expr.typ.clone()).size());              
                let call_stmt = LIRStatement::Call { dest: LIRPlace::VReg(res_vreg.clone()), func: id, args: arg_opnds};
                (Operand::Register(res_vreg), [arg_stmt_coll.into_iter().flatten().collect(), vec![call_stmt]].concat())
            }
            HIRExpressionKind::BoolTrue => {
                (Operand::BoolTrue, Vec::new())                 
            }
            HIRExpressionKind::BoolFalse => {
                (Operand::BoolFalse, Vec::new())                 
            }
            HIRExpressionKind::FieldAccess { expr, field } => {
                let LayoutInfo::Struct { size, field_offsets } = self.layouts.get_layout(expr.typ.clone()) else {
                    unreachable!();
                };
                let f_offset = field_offsets[&field].clone();

                let (expr_opnd, expr_stmts) = self.lower_expr_into_opnd(*expr); 
                
                let (expr_base, expr_offset) = match expr_opnd {
                    Operand::Register(expr_vreg) => (expr_vreg,0),
                    Operand::Deref { base, offset } => (base, offset),
                    _ => {unreachable!();},
                };
                (Operand::Deref { base: expr_base, offset: expr_offset + f_offset}, expr_stmts)
            }
            HIRExpressionKind::StructLiteral {..} => {
                let res_vreg = self.add_vreg(self.layouts.get_layout(expr.typ.clone()).size());              
                let stmts = self.lower_expr_into_target(expr, LIRPlace::VReg(res_vreg.clone())); 
                (Operand::Register(res_vreg), stmts)
            }
        }
    }
    
    fn add_vreg(&mut self, size: usize) -> VRegId {
        // TODO: later add Vreginfo stuff
        let new_id = VRegId(self.vreg_counter);
        self.vreg_counter = self.vreg_counter + 1;
        self.curr_vreg_coll.insert(new_id.clone(), VRegInfo { size, align: 8});
        new_id
    }
    
    fn get_new_blockid(&mut self) -> BlockId {
        let block_id = BlockId(self.block_counter); 
        self.block_counter = self.block_counter + 1;
        block_id 
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
        
        let TypeConstructor::Struct{fields} = deriv_typ;

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

