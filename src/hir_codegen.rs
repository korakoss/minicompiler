use std::collections::HashMap;

use crate::common::*;
use crate::hir::*;


pub struct HIRCompiler {
    output: String,
    hir_program: HIRProgram,
    jumplabel_counter: usize,
}


impl HIRCompiler {

    pub fn new(hir_program: HIRProgram) -> Self {
        HIRCompiler{
            output: String::new(), 
            hir_program: hir_program,
            jumplabel_counter: 0,
        }
    }
    
    pub fn compile(&mut self) -> String {
        
        self.emit(".global main");
        self.emit(".extern printf");
        self.emit(".align 4");
        self.emit(".data");
        self.emit(r#"fmt: .asciz "%d\n""#);
        self.emit(".text");
        
        for (f_id, func) in self.hir_program.functions.clone() {
            self.compile_function(f_id, func);
        }
        
        if let Some(funcid) = self.hir_program.entry {
            self.emit("main:");
            self.emit("    push {lr}");
            let flabel = format!("func_{}", funcid.0);
            self.emit(&format!("    bl {}", flabel));
            self.emit("    pop {lr}");
            self.emit("    bx lr");
        }
        self.output.clone()
    }
    
    // WHAT THE FUCK IS THIS SIGNATURE?
    fn compile_function(&mut self, id: FuncId, function: HIRFunction) {

        let flabel = format!("func_{}", id.0);
        let retlabel = format!("ret_{}", id.0);
    
        let HIRFunction{args, body, variables, ret_type} = function;

        // TODO: incorporate size stuff later 
        let stack_offsets: HashMap<VarId, usize> = variables.keys().enumerate().map(|(i,x)| (x.clone(), 8*(i+1))).collect();  
        let reserved_stack = 8 * variables.len(); 
        if args.len() > 4 {
                panic!("Only up to 4 args supported at the moment");
        }

        self.emit_prologue(flabel, reserved_stack);

        for i in 0..args.len(){
                self.emit(&format!("    str r{}, [fp, #-{}]", i, (i+1)*8)); 
        }
        let mut block_info = BlockInfo {
            stack_offsets: stack_offsets,
            looplabel_stack: Vec::new(),
            ret_label: retlabel.clone(),
        };

        self.compile_block(body, &mut block_info); 
        
        self.emit(&format!("{}:", retlabel));
        self.emit_epilogue(reserved_stack);
    }
    
    fn compile_block(&mut self, statements: Vec<HIRStatement>, block_info: &mut BlockInfo) {
        for stmt in statements {
            self.compile_statement(stmt, block_info);
        }
    }

            
    fn compile_statement(&mut self, statement: HIRStatement, block_info: &mut BlockInfo) {
        match statement {
            HIRStatement::Let{var: target, value} | HIRStatement::Assign { target, value } => {
                self.compile_expression(value.expr, block_info);
                let Place::Variable(var_id) = target else {panic!("Unexpected Let lvalue")};
                let var_offset = block_info.stack_offsets.get(&var_id).unwrap();
                self.emit(&format!("    str r0, [fp, #-{}]", var_offset));
            }
            HIRStatement::While { condition, body } => {
                let jlabel = self.get_jumplabel();
                let start_label = format!("while_start_{}", jlabel);
                let end_label = format!("while_end_{}", jlabel);
                
                self.emit(&format!("{}:", start_label.clone()));
                self.compile_expression(condition.expr, block_info);
                self.emit("    cmp r0, #0");
                self.emit(&format!("    beq {}", end_label.clone()));
                
                block_info.looplabel_stack.push(jlabel);
                
                self.compile_block(body, block_info);
                self.emit(&format!("    b {}", start_label));
                self.emit(&format!("{}:", end_label));

                block_info.looplabel_stack.pop();
            }
            HIRStatement::If {condition, if_body, else_body} => {
                let jlabel = self.get_jumplabel();
                let end_label = format!("branching_end_{}", jlabel);
                
                self.compile_expression(condition.expr, block_info);
                self.emit("    cmp r0, #0");

                match else_body {
                    None => {
                        self.emit(&format!("    beq {}", end_label));
                        self.compile_block(if_body, block_info);
                    }
                    Some(else_block) => {
                        let else_start_label = format!("else_start_{}", jlabel);
                        self.emit(&format!("    beq {}", else_start_label));
                        self.compile_block(if_body, block_info); 
                        self.emit(&format!("    b {}", end_label));
                        self.emit(&format!("{}:", else_start_label));
                        self.compile_block(else_block, block_info); 
                    }
                }
                self.emit(&format!("{}:",end_label));
            }
            // TODO: if
            HIRStatement::Break => {
                let curr_loop_end = format!("while_end_{}", block_info.looplabel_stack.last().unwrap());
                self.emit(&format!("    b {}", curr_loop_end));
            }
            HIRStatement::Continue=> {
                let curr_loop_start = format!("while_start_{}", block_info.looplabel_stack.last().unwrap());
                self.emit(&format!("    b {}", curr_loop_start));
            }
            HIRStatement::Return(expr) => {
                self.compile_expression(expr.expr, block_info);
                self.emit(&format!("    b {}", block_info.ret_label));
            }
            HIRStatement::Print(expr) => {
                self.compile_expression(expr.expr, block_info);
                self.emit("    mov r1, r0");
                self.emit("    ldr r0, =fmt");
                self.emit("    bl printf");
            }
        }
    }

    
    fn compile_expression(&mut self, expression: HIRExpressionKind, block_info: &BlockInfo) {
        match expression {
            HIRExpressionKind::BinOp {op, left, right} => {
                self.compile_binop(op, left.expr, right.expr, block_info); 
            }
            HIRExpressionKind::Variable(varid) => {
                self.emit(&format!("    ldr r0, [fp, #-{}]", block_info.stack_offsets.get(&varid).unwrap()));     
            }
            HIRExpressionKind::IntLiteral(n) => {
                self.emit(&format!("    ldr r0, ={}", n));   
            }
            HIRExpressionKind::FuncCall{funcid:FuncId(funcid), args:args} => {
                for arg in args.clone() {
                    self.compile_expression(arg.expr, block_info); 
                    self.emit("    push {r0}");
                }
                for i in 0..args.len() {
                    let register = format!("r{}", args.len()-i-1);
                    self.emit(&format!("    pop {{{}}}", register));    
                }
                
                // TODO: could do this more professionally
                let flabel = format!("func_{}", funcid);
                self.emit(&format!("    bl {}", flabel));
            }
            HIRExpressionKind::BoolTrue => {
                self.emit(&"    ldr r0, =1");   
            }
            HIRExpressionKind::BoolFalse => {
                self.emit(&"    ldr r0, =0");   
            }
        }
    }

    fn compile_binop(&mut self, op: BinaryOperator, left: HIRExpressionKind, right:HIRExpressionKind, scope: &BlockInfo) {
        self.compile_expression(left, scope);
        self.emit("    push {r0}");
        self.compile_expression(right, scope);
        self.emit("    pop {r1}");

        match op {
            BinaryOperator::Add => {
                self.emit("    add r0, r1, r0");
            }
            BinaryOperator::Sub => {
                self.emit("    sub r0, r1, r0");
            }
            BinaryOperator::Mul => {
                self.emit("    mul r0, r1, r0");   
            }
            BinaryOperator::Equals => {
                self.emit("    cmp r1, r0");
                self.emit("    mov r0, #0");
                self.emit("    moveq r0, #1");
            }
            BinaryOperator::Less=> {
                self.emit("    cmp r1, r0");
                self.emit("    mov r0, #0");
                self.emit("    movlt r0, #1");
            }
            BinaryOperator::Modulo => {
                self.emit("    sdiv r2, r1, r0"); 
                self.emit("    mul r2, r0, r2"); 
                self.emit("    sub r0, r1, r2");
            }
        }
    }
    
    fn emit(&mut self, line: &str) {        
        self.output.push_str(line);
        self.output.push('\n');
    }
    
    fn emit_prologue(&mut self, label: String, reserved_stack: usize) {
        self.emit(&format!("{}:", label));
        self.emit("    push {fp, lr}");     
        self.emit("    mov fp, sp");     
        self.emit(&format!("    sub sp, sp, #{}", reserved_stack)); 
    }

    fn emit_epilogue(&mut self, reserved_stack: usize) {
        self.emit(&format!("    add sp, sp, #{}", reserved_stack));         
        self.emit("    pop {fp, lr}");
        self.emit("    bx lr");
    }
    
    fn get_jumplabel(&mut self) -> String {
        let label = format!("{}", self.jumplabel_counter);
        self.jumplabel_counter = self.jumplabel_counter + 1;
        label
    }
} 



struct BlockInfo {
    looplabel_stack: Vec<String>,   
    stack_offsets: HashMap<VarId, usize>,
    ret_label: String,
}
