use std::collections::HashMap;
use crate::{ast::BinaryOperator, common::VariableInfo, hir::*};


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
        
        // TODO: ugh. Don't clone
        for (f_id, func) in self.hir_program.functions.clone(){
            self.compile_function(f_id, func);
        }

        let globscope_id = self.hir_program.global_scope.expect("Expected global statements");
        let globvars = self.hir_program.collect_scope_vars(&globscope_id);
        let stack_offsets: HashMap<VarId, usize> = globvars.values().enumerate().map(|(i,x)| (x.clone(), 8*i)).collect();  // TODO: incorporate size stuff later 
        let globscope_info = ScopeInfo{
            var_offsets: stack_offsets, 
            curr_loop_start: None,
            curr_loop_end: None, 
            curr_func_epi: None, 
        };
        let reserved_stack = 8 * globvars.len();       // TODO: incorporate size stuff later 
        self.emit_prologue("main".to_string(), reserved_stack); 
        
        // TODO: compile global block or sth                
        self.compile_block(&globscope_id, &globscope_info);
        // Epilogue
        self.emit_epilogue(reserved_stack);                    // TODO: change
        
        self.output.clone() 
    }
    
    // WHAT THE FUCK IS THIS SIGNATURE?
    fn compile_function(&mut self, id: FuncId, function: HIRFunction) {

        let flabel = format!("func_{}", id.0);
        let retlabel = format!("ret_{}", id.0);
    
        let HIRFunction{args, body, ret_type} = function;

        let vars = self.hir_program.collect_scope_vars(&body);
        let stack_offsets: HashMap<VarId, usize> = vars.values().enumerate().map(|(i,x)| (x.clone(), 8*(i+1))).collect();  // TODO: incorporate size stuff later 
        if args.len() > 4 {
                panic!("Only up to 4 args supported at the moment");
        }
        let reserved_stack = 8 * vars.len();       // TODO: incorporate size stuff later 

        self.emit_prologue(flabel, reserved_stack);

        for i in 0..args.len(){
                self.emit(&format!("    str r{}, [fp, #-{}]", i, (i+1)*8)); 
        }

        let func_scope = ScopeInfo{
            var_offsets: stack_offsets, 
            curr_loop_start: None,
            curr_loop_end: None, 
            curr_func_epi: Some(retlabel.clone())
        };
        self.compile_block(&body, &func_scope); 
        
        self.emit(&format!("{}:", retlabel));
        self.emit_epilogue(reserved_stack);
    }

    fn get_jumplabel(&mut self) -> String {
        let label = format!("{}", self.jumplabel_counter);
        self.jumplabel_counter = self.jumplabel_counter + 1;
        label
    }
        
    fn compile_statement(&mut self, statement: HIRStatement, scope: &ScopeInfo) {
        match statement {
            HIRStatement::Let{var, value} => {
                self.compile_expression(value.expr, scope);
                // TODO: eek! this sucks. refactor!
                if let Place::Variable(varid) = var {
                    let var_offset = scope.var_offsets.get(&varid).unwrap();
                    self.emit(&format!("    str r0, [fp, #-{}]", var_offset));
                } else {
                    panic!("Unexpected Let lvalue");
                }
            }
            HIRStatement::Assign { target, value } => {
                self.compile_expression(value.expr, scope);
                // TODO: eek! this sucks. refactor!
                if let Place::Variable(varid) = target {
                    let var_offset = scope.var_offsets.get(&varid).unwrap();
                    self.emit(&format!("    str r0, [fp, #-{}]", var_offset));
                } else {
                    panic!("Unexpected Let lvalue");
                }

            }
            HIRStatement::While { condition, body } => {
                let jlabel = self.get_jumplabel();
                let start_label = format!("while_start_{}", jlabel);
                let end_label = format!("while_end_{}", jlabel);
                
                self.emit(&format!("{}:", start_label.clone()));
                self.compile_expression(condition.expr, scope);
                self.emit("    cmp r0, #0");
                self.emit(&format!("    beq {}", end_label.clone()));
                
                let mut block_scope = scope.clone();
                block_scope.curr_loop_start = Some(start_label.clone());
                block_scope.curr_loop_end = Some(end_label.clone());
                
                self.compile_block(&body, &block_scope);
                
                self.emit(&format!("    b {}", start_label));
                self.emit(&format!("{}:", end_label));
            }
            HIRStatement::If {condition, if_body, else_body} => {
                let jlabel = self.get_jumplabel();
                let end_label = format!("branching_end_{}", jlabel);
                
                self.compile_expression(condition.expr, scope);
                self.emit("    cmp r0, #0");

                match else_body {
                    None => {
                        self.emit(&format!("    beq {}", end_label));
                        self.compile_block(&if_body, scope);
                    }
                    Some(else_block) => {
                        let else_start_label = format!("else_start_{}", jlabel);
                        self.emit(&format!("    beq {}", else_start_label));
                        self.compile_block(&if_body, scope); 
                        self.emit(&format!("    b {}", end_label));
                        self.emit(&format!("{}:", else_start_label));
                        self.compile_block(&else_block, scope); 
                    }
                }
                self.emit(&format!("{}:",end_label));
            }
            // TODO: if
            HIRStatement::Break => {
                self.emit(&format!("    b {}", scope.curr_loop_end.clone().unwrap()));
            }
            HIRStatement::Continue=> {
                self.emit(&format!("    b {}", scope.curr_loop_start.clone().unwrap()));
            }
            HIRStatement::Return(expr) => {
                self.compile_expression(expr.expr, scope);
                self.emit(&format!("    b {}", scope.curr_func_epi.clone().unwrap()));
            }
            HIRStatement::Print(expr) => {
                self.compile_expression(expr.expr, scope);
                self.emit("    mov r1, r0");
                self.emit("    ldr r0, =fmt");
                self.emit("    bl printf");
            }
        }
    }

    fn compile_block(&mut self, blockid: &ScopeId, block_scope: &ScopeInfo) {
        let block = self.hir_program.scopes.get(&blockid).unwrap().clone(); 
        for stmt in block.statements {
            self.compile_statement(stmt, block_scope);
        }
    }

    fn compile_expression(&mut self, expression: TypedExpressionKind, scope: &ScopeInfo) {
        match expression {
            TypedExpressionKind::BinOp {op, left, right} => {
                self.compile_binop(op, left.expr, right.expr, scope); 
            }
            TypedExpressionKind::Variable(varid) => {
                let var_offset = scope.var_offsets.get(&varid).unwrap();
                self.emit(&format!("    ldr r0, [fp, #-{}]", var_offset));     
            }
            TypedExpressionKind::IntLiteral(n) => {
                self.emit(&format!("    ldr r0, ={}", n));   
            }
            TypedExpressionKind::FuncCall{funcid:FuncId(funcid), args:args} => {
                for arg in args.clone() {
                    self.compile_expression(arg.expr, scope); 
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
        }
    }

    fn compile_binop(&mut self, op: BinaryOperator, left: TypedExpressionKind, right:TypedExpressionKind, scope: &ScopeInfo) {
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

}


#[derive(Clone)]
struct ScopeInfo {
    var_offsets: HashMap<VarId, usize>,
    curr_loop_start: Option<String>,          // label for the end of innermost active loop
    curr_loop_end: Option<String>,          
    curr_func_epi: Option<String>,
}
